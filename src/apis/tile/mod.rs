//! 幾つかのタイルデータを公開

mod glb;

use std::io::Cursor;

use axum::async_trait;
use axum::extract::Host;
use axum::http::Method;
use axum_extra::extract::CookieJar;
use coordinate_transformer::{pixel2ll, pixel_resolution};
use gltf::Glb;
use neo4rs::{query, BoltFloat, BoltPoint2D};
use openapi::apis::tile::{Tile, TilesLandZxyGetResponse, TilesWaterZxyGetResponse};
use openapi::models::{TilesLandZxyGetPathParams, TilesWaterZxyGetPathParams};
use openapi::types::ByteArray;
use reqwest::Client;
use spade::{DelaunayTriangulation, FloatTriangulation, HasPosition, Point2, Triangulation};
use voxel_tiler_core::coordinate_transformer::ZoomLv;
use voxel_tiler_core::giaj_terrain::{AltitudeResolutionCriteria, GIAJTerrainImageSampler};
use voxel_tiler_core::glb::{GlbGen, Mime, TextureInfo};
use voxel_tiler_core::image::{DynamicImage, ImageFormat, ImageReader};
use voxel_tiler_core::mesh::{Mesher, ValidSide};

use crate::apis::tile::glb::{Point3D, VMesh, WaterGlbGen};
use crate::apis::ServerImpl;
use crate::cache::items::{CacheDataType, CacheKey, CachedData, TileId};

async fn fetch_image(url: &str, http_client: Client) -> anyhow::Result<DynamicImage> {
    let result = http_client.get(url).send().await?;
    let bytes = result.bytes().await?;
    let cursor = Cursor::new(bytes);
    let img = ImageReader::new(cursor).with_guessed_format()?.decode()?;
    Ok(img)
}

async fn generate_land_tile(path_params: TilesLandZxyGetPathParams, http_client: Client) -> Result<Vec<u8>, String> {
    let TilesLandZxyGetPathParams { x, y, z } = path_params;
    let zoom_lv = ZoomLv::parse(z).map_err(|_| "Invalid zoom level")?;

    let dem = fetch_image(
        &format!("https://tiles.gsj.jp/tiles/elev/land/{z}/{y}/{x}.png"),
        http_client.clone(),
    )
        .await
        .map_err(|e| {
            e.to_string()
        })?
        .flipv();

    let photo = {
        let dimage = fetch_image(
            &format!("https://cyberjapandata.gsi.go.jp/xyz/seamlessphoto/{z}/{x}/{y}.jpg"),
            http_client.clone(),
        )
            .await
            .map_err(|e| {
                e.to_string()
            })?
            .flipv();

        let mut buf = Vec::<u8>::new();
        dimage.write_to(&mut Cursor::new(&mut buf), ImageFormat::Jpeg).unwrap();

        buf
    };

    let resolution = {
        let (pixel_x, pixel_y) = (x as u32 * 256 + 128, y as u32 * 256 + 128);
        let (_long, lat) = pixel2ll((pixel_x, pixel_y), ZoomLv::parse(z).unwrap());
        AltitudeResolutionCriteria::Lat(lat, zoom_lv)
    };

    let sampled = GIAJTerrainImageSampler::sampling(resolution, dem, None).unwrap();

    let mesh = Mesher::meshing(
        sampled,
        ValidSide::all() - ValidSide::BORDER - ValidSide::BOTTOM,
    )
        .simplify();

    let texture = TextureInfo {
        buf: Some(photo),
        uri: None,
        mime_type: Mime::ImageJpeg,
    };

    let glb = Glb::from_voxel_mesh_with_texture_projected_z(mesh, texture).unwrap();

    Ok(glb.to_vec().unwrap())
}

#[async_trait]
impl Tile for ServerImpl {
    async fn tiles_land_zxy_get(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
        path_params: TilesLandZxyGetPathParams,
    ) -> Result<TilesLandZxyGetResponse, String> {
        let TilesLandZxyGetPathParams { x, y, z } = path_params;

        let cache_key = CacheKey {
            data_type: CacheDataType::Land,
            tile_id: TileId::new(x as u32, y as u32, z as u8),
        };


        let compute_fu = async {
            let bates = generate_land_tile(path_params, self.http_client.clone()).await;
            CachedData::new(bates.unwrap_or_default())
        };

        let buf = self.cache.get_or_compute(cache_key, compute_fu)
            .await
            .bytes;


        let response = TilesLandZxyGetResponse::Status200 {
            body: ByteArray(buf),
            content_encoding: None,
        };

        Ok(response)
    }

    async fn tiles_water_zxy_get(
        &self,
        _method: Method,
        _host: Host,
        _cookies: CookieJar,
        path_params: TilesWaterZxyGetPathParams,
    ) -> Result<TilesWaterZxyGetResponse, String> {
        let TilesWaterZxyGetPathParams { z, x, y } = path_params;

        let tile_path = calc_tile_path(z as u32, x as u32, y as u32);

        let query = query(
            &format!(r#"
MATCH {tile_path}-[:MEMBER]->(n:RiverNode)-[:WATER_LEVEL]->(wl:WaterLevel)
RETURN n.location AS location, wl.value AS water_level
            "#)
        );

        let mut result = self.graph.execute(query)
            .await
            .unwrap();

        let mut nodes = Vec::<RiverNode>::new();

        while let Ok(Some(row)) = result.next().await {
            let location: BoltPoint2D = row.get("location").unwrap();
            let water_level: BoltFloat = row.get("water_level").unwrap();

            println!("{:?}, {:?}", location, water_level);

            nodes.push(RiverNode {
                location: Point2::new(location.x.value, location.y.value),
                water_level: water_level.value,
            });
        }

        println!("{:?}", nodes);

        //let data = generate_warter_surfce_data(nodes, x as u32, y as u32, z as u32);
        let data = gen_poly(nodes, x as u32, y as u32, z as u32);

        Ok(TilesWaterZxyGetResponse::Status200 {
            body: ByteArray(data),
            content_encoding: None,
        })
    }
}

// あるタイル座標までのレベル0のタイルからのパスを計算
fn calc_tile_path(z: u32, x: u32, y: u32) -> String {
    (0..=z).map(|current_z| {
        // 算術演算で求める
        let x = x >> (z - current_z);
        let y = y >> (z - current_z);
        format!("(:Tile{current_z}{{x:{x},y:{y}}})")
    }).collect::<Vec<_>>()
        .join("-[:CHILD]->")
}

#[derive(Debug)]
struct RiverNode {
    location: Point2<f64>,
    water_level: f64,
}

impl HasPosition for RiverNode {
    type Scalar = f64;

    fn position(&self) -> Point2<f64> {
        self.location
    }
}

fn gen_poly(nodes: Vec<RiverNode>, tile_x: u32, tile_y: u32, zoom: u32) -> Vec<u8> {
    let mut t = DelaunayTriangulation::<RiverNode>::bulk_load(nodes).unwrap();
    let b = t.barycentric();

    let res = {
        let (_long, lat) = pixel2ll((tile_x * 256, tile_y * 256), ZoomLv::parse(zoom).unwrap());

        pixel_resolution(lat, ZoomLv::parse(zoom).unwrap()) as f32
    };


    let query_points = {
        let left_top = (tile_x * 256, tile_y * 256);
        let right_top = ((tile_x + 1) * 256, tile_y * 256);
        let right_bottom = ((tile_x + 1) * 256, (tile_y + 1) * 256);
        let left_bottom = (tile_x * 256, (tile_y + 1) * 256);

        vec![left_top.clone(), right_top.clone(), right_bottom.clone(), left_bottom.clone()]
    };

    let points = query_points.iter().map(|point| {
        let ll = pixel2ll((point.0 as u32, point.1 as u32), ZoomLv::parse(zoom).unwrap());
        let ll_point = Point2::new(ll.0.to_degrees(), ll.1.to_degrees());

        let water_level = b.interpolate(|v| v.data().water_level, ll_point).unwrap_or_default() as f32;

        (point, water_level)
    }).collect::<Vec<_>>();


    let points = [
        Point3D::new([0., 0., points[0].1]),
        Point3D::new([res, 0., points[1].1]),
        Point3D::new([res, res, points[2].1]),
        Point3D::new([0., res, points[3].1]),
    ];

    let vmesh = VMesh::create_water_surface(points);


    Glb::from_vmesh(vmesh).unwrap().to_vec().unwrap()
}


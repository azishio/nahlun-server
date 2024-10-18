//! 幾つかのタイルデータを公開

use std::io::Cursor;

use axum::async_trait;
use axum::extract::Host;
use axum::http::Method;
use axum_extra::extract::CookieJar;
use coordinate_transformer::pixel2ll;
use neo4rs::query;
use openapi::apis::tile::{Tile, TilesLandZxyGetResponse, TilesWaterZxyGetResponse};
use openapi::models::{TilesLandZxyGetPathParams, TilesWaterZxyGetPathParams};
use openapi::types::ByteArray;
use reqwest::Client;
use voxel_tiler_core::coordinate_transformer::ZoomLv;
use voxel_tiler_core::giaj_terrain::{AltitudeResolutionCriteria, GIAJTerrainImageSampler};
use voxel_tiler_core::glb::{Glb, GlbGen, Mime, TextureInfo};
use voxel_tiler_core::image::{DynamicImage, ImageFormat, ImageReader};
use voxel_tiler_core::mesh::{Mesher, ValidSide};

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
        todo!()
    }
}

//! 幾つかのタイルデータを公開

use std::io::Cursor;

use axum::async_trait;
use axum::extract::Host;
use axum::http::Method;
use axum_extra::extract::CookieJar;
use coordinate_transformer::pixel2ll;
use openapi::apis::tile::{Tile, TilesLandZxyGetResponse, TilesWaterZxyGetResponse};
use openapi::models::{TilesLandZxyGetPathParams, TilesWaterZxyGetPathParams};
use openapi::types::ByteArray;
use reqwest::Client;
use voxel_tiler_core::coordinate_transformer::ZoomLv;
use voxel_tiler_core::giaj_terrain::{AltitudeResolutionCriteria, GIAJTerrainImageSampler};
use voxel_tiler_core::glb::{Glb, GlbGen, Mime, TextureInfo};
use voxel_tiler_core::image::{DynamicImage, ImageReader};
use voxel_tiler_core::mesh::{Mesher, ValidSide};

use crate::apis::ServerImpl;

async fn fetch_image(url: &str, http_client: Client) -> anyhow::Result<DynamicImage> {
    let result = http_client.get(url).send().await?;
    let bytes = result.bytes().await?;
    let cursor = Cursor::new(bytes);
    let img = ImageReader::new(cursor).with_guessed_format()?.decode()?;
    Ok(img)
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
        let TilesLandZxyGetPathParams { z, x, y } = path_params;

        let zoom_lv = ZoomLv::parse(z).map_err(|_| "Invalid zoom level")?;

        let dem = fetch_image(
            &format!("https://tiles.gsj.jp/tiles/elev/land/{z}/{y}/{x}.png"),
            self.http_client.clone(),
        )
        .await
        .map_err(|e| e.to_string())?;

        let photo = self
            .http_client
            .get(&format!(
                "https://cyberjapandata.gsi.go.jp/xyz/seamlessphoto/{z}/{x}/{y}.jpg"
            ))
            .send()
            .await
            .map_err(|e| e.to_string())?
            .bytes()
            .await
            .map_err(|e| e.to_string())?;

        let resolution = {
            let (_long, lat) = pixel2ll((x as u32, y as u32), ZoomLv::parse(z * 256).unwrap());
            AltitudeResolutionCriteria::Lat(lat, zoom_lv);
            AltitudeResolutionCriteria::ZoomLv(zoom_lv)
        };

        let sampled = GIAJTerrainImageSampler::sampling(resolution, dem, None).unwrap();

        let mesh = Mesher::meshing(
            sampled,
            ValidSide::all() - ValidSide::BORDER - ValidSide::BOTTOM,
        )
        .simplify();

        let texture = TextureInfo {
            buf: Some(photo.to_vec()),
            uri: None,
            mime_type: Mime::ImageJpeg,
        };

        let glb = Glb::from_voxel_mesh_with_texture_projected_z(mesh, texture)
            .map_err(|e| e.to_string())?;

        let buf = glb.to_vec().map_err(|e| e.to_string())?;

        let response = TilesLandZxyGetResponse::Status200 {
            body: ByteArray(buf),
            content_encoding: None,
        };

        println!("created tile: z={}, x={}, y={}", z, x, y);
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

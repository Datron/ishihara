mod image_gen;

use std::io::Cursor;

use axum::{
    response::{Html, IntoResponse},
    routing::{get, post},
    Json, Router,
};
use image_gen::{ishihara_plate, ColourBlindnessType};
use serde::Deserialize;
use tower_service::Service;
use tracing_subscriber::{
    fmt::{format::Pretty, time::UtcTime},
    layer::SubscriberExt,
    util::SubscriberInitExt,
};
use tracing_web::{performance_layer, MakeConsoleWriter};
use worker::*;

#[derive(Deserialize)]
struct IshiharaImageRequest {
    pub message: String,
    pub height: u32,
    pub width: u32,
    // pub font_size: f32,
    pub colourblind_mode: ColourBlindnessType,
}

// Multiple calls to `init` will cause a panic as a tracing subscriber is already set.
// So we use the `start` event to initialize our tracing subscriber when the worker starts.
#[event(start)]
fn start() {
    let fmt_layer = tracing_subscriber::fmt::layer()
        .json()
        .with_ansi(false) // Only partially supported across JavaScript runtimes
        .with_timer(UtcTime::rfc_3339()) // std::time is not available in browsers
        .with_writer(MakeConsoleWriter); // write events to the console
    let perf_layer = performance_layer().with_details_from_fields(Pretty::default());
    tracing_subscriber::registry()
        .with(fmt_layer)
        .with(perf_layer)
        .init();
}

#[event(fetch)]
async fn main(
    req: HttpRequest,
    _env: Env,
    _ctx: Context,
) -> Result<axum::http::Response<axum::body::Body>> {
    Ok(router().call(req).await?)
}

fn router() -> Router {
    Router::new()
        .route("/", get(home_page))
        .route("/image_gen", post(generate_ishihara_image))
}

async fn home_page() -> Html<&'static str> {
    Html(include_str!("index.html"))
}

async fn generate_ishihara_image(
    Json(payload): Json<IshiharaImageRequest>,
) -> std::result::Result<impl IntoResponse, Html<String>> {
    let img = ishihara_plate(
        &payload.message,
        payload.width,
        payload.height,
        payload.colourblind_mode,
    );

    let mut bytes: Vec<u8> = Vec::new();
    img.write_to(&mut Cursor::new(&mut bytes), image::ImageFormat::Png)
        .map_err(|err| Html(err.to_string()))?;
    Ok(([("content-type", "image/png")], bytes))
}

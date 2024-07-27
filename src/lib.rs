mod image_gen;

use std::io::Cursor;

use image_gen::ishihara_plate;
use serde_json::{Map, Value};
use tracing_subscriber::{
    fmt::{format::Pretty, time::UtcTime},
    layer::SubscriberExt,
    util::SubscriberInitExt,
};
use tracing_web::{performance_layer, MakeConsoleWriter};
use worker::*;

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
async fn main(req: Request, _env: Env, _ctx: Context) -> Result<Response> {
    let query_map = req.query::<Map<String, Value>>().unwrap_or_default();
    let text = query_map.get("text").map(|e| e.as_str().unwrap_or("hello")).unwrap_or("hello");
    let img = ishihara_plate(text);

    let mut bytes: Vec<u8> = Vec::new();
    img.write_to(&mut Cursor::new(&mut bytes), image::ImageFormat::Png)
        .map_err(|err| Error::RustError(err.to_string()))?;
    let mut response = Response::from_bytes(bytes)?;
    response.headers_mut().set("Content-Type", "image/png")?;
    Ok(response)
}

use std::io::Cursor;

use ab_glyph::{FontRef, PxScale};
use image::{Rgb, RgbImage};
use imageproc::{drawing::{draw_filled_circle_mut, draw_filled_rect_mut, draw_text_mut}, rect::Rect};
use rand::Rng;
use tracing_subscriber::{
    fmt::{format::Pretty, time::UtcTime},
    layer::SubscriberExt,
    util::SubscriberInitExt,
};
use tracing_web::{performance_layer, MakeConsoleWriter};
use worker::*;

#[derive(Debug, Clone)]
struct ColourPallete {
    pub red: Vec<Rgb<u8>>,
    pub green: Vec<Rgb<u8>>,
    // pub blue: Vec<Rgb<u8>>,
}

#[derive(Debug)]
enum ColourBlindnessType {
    RedGreen,
}

fn get_colour_palette(ctype: ColourBlindnessType) -> ColourPallete {
    match ctype {
        ColourBlindnessType::RedGreen => ColourPallete {
            red: vec![Rgb([220, 134, 127])],
            green: vec![
                Rgb([185, 184, 98]),
                Rgb([207, 207, 131]),
                Rgb([160, 148, 82]),
            ],
            // blue: vec![],
        },
    }
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
async fn main(req: Request, _env: Env, _ctx: Context) -> Result<Response> {
    let canvas_width = 800;
    let canvas_height = 400;
    let mut img = RgbImage::new(canvas_width, canvas_height);
    draw_filled_rect_mut(&mut img, Rect::at(0, 0).of_size(canvas_width, canvas_height), Rgb([255, 255, 255]));

    let canvas_width = canvas_width as i32;
    let canvas_height = canvas_height as i32;

    let colour_palette = get_colour_palette(ColourBlindnessType::RedGreen);
    let circle_min_radius: i32 = 6;
    let circle_max_radius: i32 = 10;
    let mut generator = rand::thread_rng();

    for y in (0..=canvas_height).step_by((generator.gen_range(circle_min_radius..circle_max_radius) * 2) as usize) {
        let mut x = 0;
        loop {
            let radius = generator.gen_range(circle_min_radius..circle_max_radius) as i32;
            x += radius;
            let index = generator.gen_range(0..colour_palette.green.len());
            draw_filled_circle_mut(&mut img, (x, y), radius, colour_palette.green[index]);
            // tracing::info!("current position: {x}");
            x += radius;
            // tracing::info!("radius: {radius} and next position: {x}");
            if x >= canvas_width {
                break;
            }
        }
    }

    let font = FontRef::try_from_slice(include_bytes!("assets/Nano.ttf")).unwrap();

    let height = 25.0;
    let scale = PxScale {
        x: height * 2.7,
        y: height * 4.0,
    };
    let url = req.url()?;
    let text = url.query().unwrap_or("Hello world!".into());
    draw_text_mut(&mut img, colour_palette.red[0], 40, 130, scale, &font, text);

    let mut bytes: Vec<u8> = Vec::new();
    img.write_to(&mut Cursor::new(&mut bytes), image::ImageFormat::Png)
        .map_err(|err| Error::RustError(err.to_string()))?;
    let mut response = Response::from_bytes(bytes)?;
    response.headers_mut().set("Content-Type", "image/png")?;
    Ok(response)
}

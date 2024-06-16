use std::io::Cursor;

use ab_glyph::{FontRef, PxScale};
use image::{Rgb, RgbImage};
use imageproc::drawing::draw_text_mut;
use worker::*;

#[event(fetch)]
async fn main(_req: Request, _env: Env, _ctx: Context) -> Result<Response> {
    let mut img = RgbImage::new(400, 400);

    for x in 190..=210 {
        for y in 190..210 {
            img.put_pixel(x, y, Rgb([255, 0, 0]));
            img.put_pixel(y, x, Rgb([255, 0, 0]));
        }
    }

    let font = FontRef::try_from_slice(include_bytes!("assets/Nano.ttf")).unwrap();

    let height = 35.0;
    let scale = PxScale {
        x: height * 2.0,
        y: height,
    };

    let text = "Hello, world!";
    draw_text_mut(&mut img, Rgb([255, 255, 255]), 0, 0, scale, &font, text);

    let mut bytes: Vec<u8> = Vec::new();
    img.write_to(&mut Cursor::new(&mut bytes), image::ImageFormat::Png)
        .map_err(|err| Error::RustError(err.to_string()))?;
    let mut response = Response::from_bytes(bytes)?;
    response.headers_mut().set("Content-Type", "image/png")?;
    Ok(response)
}

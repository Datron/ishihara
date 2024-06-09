use std::io::Cursor;

use image::{Rgb, RgbImage};
use worker::*;

#[event(fetch)]
async fn main(req: Request, env: Env, ctx: Context) -> Result<Response> {
    let mut img = RgbImage::new(32, 32);

    for x in 15..=17 {
        for y in 8..24 {
            img.put_pixel(x, y, Rgb([255, 0, 0]));
            img.put_pixel(y, x, Rgb([255, 0, 0]));
        }
    }
    let mut bytes: Vec<u8> = Vec::new();
    img.write_to(&mut Cursor::new(&mut bytes), image::ImageFormat::Png)
        .map_err(|err| Error::RustError(err.to_string()))?;

    let mut response = Response::from_bytes(bytes)?;
    response.headers_mut().set("Content-Type", "image/png")?;
    Ok(response)
}

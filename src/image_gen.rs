use ab_glyph::{Font, FontRef, Rect};
use image::{ImageBuffer, Rgb, RgbImage};
use imageproc::drawing::{draw_filled_circle_mut, draw_filled_rect_mut};
use rand::{rngs::ThreadRng, Rng};
use serde::Deserialize;

#[derive(Debug, Clone)]
pub(crate) struct ColourPallete {
    pub number_colours: Vec<Rgb<u8>>,
    pub background_colours: Vec<Rgb<u8>>,
}

#[derive(Debug, Deserialize)]
pub(crate) enum ColourBlindnessType {
    Protanopia,
    Deuteranopia,
    Tritanopia,
    All,
}

impl ColourBlindnessType {
    pub fn get_colour_palette(self: &ColourBlindnessType) -> ColourPallete {
        match self {
            ColourBlindnessType::Protanopia => ColourPallete {
                number_colours: vec![
                    Rgb([216, 136, 86]),
                    Rgb([238, 177, 121]),
                    Rgb([227, 178, 118]),
                ],
                background_colours: vec![
                    Rgb([146, 151, 73]),
                    Rgb([200, 203, 164]),
                    Rgb([223, 213, 121]),
                ],
            },
            ColourBlindnessType::Deuteranopia => ColourPallete {
                number_colours: vec![
                    Rgb([227, 140, 158]),
                    Rgb([189, 107, 137]),
                    Rgb([127, 74, 96]),
                ],
                background_colours: vec![
                    Rgb([96, 94, 83]),
                    Rgb([161, 158, 141]),
                    Rgb([85, 78, 62]),
                ],
            },
            ColourBlindnessType::Tritanopia => ColourPallete {
                number_colours: vec![
                    Rgb([167, 151, 56]),
                    Rgb([226, 193, 100]),
                    Rgb([255, 220, 50]),
                ],
                background_colours: vec![
                    Rgb([172, 184, 210]),
                    Rgb([195, 193, 255]),
                    Rgb([120, 148, 255]),
                ],
            },
            ColourBlindnessType::All => ColourPallete {
                number_colours: vec![
                    Rgb([147, 155, 107]),
                    Rgb([174, 168, 88]),
                    Rgb([214, 213, 123]),
                ],
                background_colours: vec![
                    Rgb([232, 130, 93]),
                    Rgb([229, 128, 90]),
                    Rgb([236, 180, 137]),
                ],
            },
        }
    }
}

#[derive(Debug)]
struct IshiharaCircle {
    pub x: i32,
    pub y: i32,
    pub radius: i32,
}

pub fn ishihara_plate(
    text: &str,
    canvas_width: u32,
    canvas_height: u32,
    mode: ColourBlindnessType,
) -> ImageBuffer<Rgb<u8>, Vec<u8>> {
    let mut img = RgbImage::new(canvas_width, canvas_height);
    draw_filled_rect_mut(
        &mut img,
        imageproc::rect::Rect::at(0, 0).of_size(canvas_width, canvas_height),
        Rgb([255, 255, 255]),
    );

    let canvas_width = canvas_width as i32;
    let canvas_height = canvas_height as i32;
    let generator = rand::thread_rng();
    let font = FontRef::try_from_slice(include_bytes!("assets/Design2.ttf")).unwrap();
    let mut bounds = (0, 0);
    for c in text.chars().into_iter() {
        let new_bounds;
        (img, new_bounds) = draw_ishihara_segment(generator.clone(), &font, img, c, bounds, &mode);
        bounds = (
            bounds.0 + new_bounds.max.x as i32 + 5,
            bounds.1 + new_bounds.max.y as i32,
        );
        if bounds.0 >= canvas_width - 100 {
            bounds = (0, bounds.1 + canvas_height / 2 - 40);
        }
    }
    img
}

fn draw_ishihara_segment(
    mut generator: ThreadRng,
    font: &FontRef,
    mut img: ImageBuffer<Rgb<u8>, Vec<u8>>,
    letter: char,
    offset: (i32, i32),
    mode: &ColourBlindnessType,
) -> (ImageBuffer<Rgb<u8>, Vec<u8>>, Rect) {
    let circle_min_radius: i32 = 4;
    let circle_max_radius: i32 = 10;
    let glyph = font.glyph_id(letter).with_scale(200.0);
    let colour_palette = mode.get_colour_palette();
    let mut circles: Vec<IshiharaCircle> = vec![];
    let mut bounds = Rect::default();
    if let Some(outline) = font.outline_glyph(glyph) {
        bounds = outline.px_bounds();
        outline.draw(|x, y, c| {
            let radius = generator.gen_range(circle_min_radius..=circle_max_radius);
            let new_x = x as i32;
            let new_y = y as i32;
            for existing_circle in circles.iter() {
                let gap_x = existing_circle.x.abs_diff(new_x) as i32;
                let gap_y = existing_circle.y.abs_diff(new_y) as i32;
                let gap = (gap_x * gap_x) + (gap_y * gap_y);
                let d = (gap as f32).sqrt() as i32;
                if gap < (existing_circle.radius * existing_circle.radius)
                    || d < (existing_circle.radius + radius)
                {
                    return;
                }
            }
            if c < 0.9 {
                let colour = generator.gen_range(0..colour_palette.background_colours.len());
                draw_filled_circle_mut(
                    &mut img,
                    (new_x + offset.0, new_y + offset.1),
                    radius,
                    colour_palette.background_colours[colour],
                );
            } else {
                let colour = generator.gen_range(0..colour_palette.number_colours.len());
                draw_filled_circle_mut(
                    &mut img,
                    (new_x + offset.0, new_y + offset.1),
                    radius,
                    colour_palette.number_colours[colour],
                );
            }
            circles.push(IshiharaCircle {
                x: new_x,
                y: new_y,
                radius: radius,
            });
        });
    }
    (img, bounds)
}

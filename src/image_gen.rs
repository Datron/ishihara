use ab_glyph::{Font, FontRef, Rect};
use image::{ImageBuffer, Rgb, RgbImage};
use imageproc::drawing::{draw_filled_circle_mut, draw_filled_rect_mut};
use rand::{rngs::ThreadRng, Rng};

#[derive(Debug, Clone)]
struct ColourPallete {
    pub number_colours: Vec<Rgb<u8>>,
    pub background_colours: Vec<Rgb<u8>>,
}

#[derive(Debug)]
enum ColourBlindnessType {
    RedGreen,
    // Green
}

#[derive(Debug)]
struct IshiharaCircle {
    pub x: i32,
    pub y: i32,
    pub radius: i32,
}

fn get_colour_palette(ctype: ColourBlindnessType) -> ColourPallete {
    match ctype {
        ColourBlindnessType::RedGreen => ColourPallete {
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
        // ColourBlindnessType::Green => ColourPallete {
        //     number_colours: vec![Rgb([219, 226, 139])],
        //     background_colours: vec![Rgb([121, 185, 38]), Rgb([153, 200, 24])],
        // },
    }
}

pub fn ishihara_plate(text: &str) -> ImageBuffer<Rgb<u8>, Vec<u8>> {
    let canvas_width = 1000;
    let canvas_height = 200;
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
        (img, new_bounds) = draw_ishihara_segment(generator.clone(), &font, img, c, bounds, ColourBlindnessType::RedGreen);
        bounds = (bounds.0 + new_bounds.max.x as i32 + 5, bounds.1 + new_bounds.max.y as i32);
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
    mode: ColourBlindnessType
) -> (ImageBuffer<Rgb<u8>, Vec<u8>>, Rect) {
    let circle_min_radius: i32 = 4;
    let circle_max_radius: i32 = 10;
    let glyph = font.glyph_id(letter).with_scale(200.0);
    let colour_palette = get_colour_palette(mode);
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

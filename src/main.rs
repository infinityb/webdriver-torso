use image::{RgbImage, Rgb};
use rand::seq::SliceRandom;
use rand::Rng;
use rusttype::{Font, Scale, point};
use font_kit::source::SystemSource;
use font_kit::family_name::FamilyName;
use font_kit::properties::Properties;
use font_kit::handle::Handle;
use std::error::Error;

const WIDTH: u32 = 768;
const HEIGHT: u32 = 512;

fn main() {
    let mut rng = rand::thread_rng();
    let slide_number = rng.gen_range(1..=10000);

    let mut img = RgbImage::from_pixel(WIDTH, HEIGHT, Rgb([255, 255, 255]));
    let include_three = rng.gen_bool(0.05);
    let num_rects = 2 + include_three as i32;
    let all_choices = &[
        Rgb([0xFF, 0x00, 0x00]),
        Rgb([0x00, 0x00, 0xFF]),
        Rgb([0xFF, 0xFF, 0x00]),
        Rgb([0x00, 0xFF, 0x00]),
    ];
    for _ in 0..num_rects {
        let endex = 2 + rng.gen_bool(0.001) as usize + rng.gen_bool(0.001) as usize;
        let this_choices = &all_choices[0..endex];
        let color = this_choices.choose(&mut rng).expect("choices must be non-empty");

        let x0 = rng.gen_range(0..WIDTH - 50);
        let y0 = rng.gen_range(0..HEIGHT - 50);
        let x1 = x0 + rng.gen_range(50..=WIDTH - x0);
        let y1 = y0 + rng.gen_range(50..=HEIGHT - y0);

        for x in x0..x1 {
            for y in y0..y1 {
                img.put_pixel(x, y, *color);
            }
        }
    }

    // Load a font
    let font = match load_system_font() {
        Ok(font) => font,
        Err(e) => {
            eprintln!("Failed to load font: {}", e);
            return;
        }
    };

    // Set up font scale and position
    let yscale = 30;
    let scale = Scale {
        x: 22.0, // yes.
        y: yscale as f32
    };
    // slide blanking
    for _ in 0..num_rects {
        for x in 0..256 {
            for y in 1..yscale {
                img.put_pixel(x, HEIGHT - y, Rgb([0xFF, 0xFF, 0xFF]));
            }
        }
    }

    let text = format!("aqua.flv - synthetic frame #{}", slide_number);
    draw_text(&mut img, Rgb([0, 0, 0]), 5, (HEIGHT - yscale) as i32, scale, &font, &text);

    match img.save("webdriver_torso_slide.png") {
        Ok(_) => println!("Image saved successfully!"),
        Err(e) => eprintln!("Failed to save image: {}", e),
    }
}

fn load_system_font() -> Result<Font<'static>, Box<dyn Error>> {
    let source = SystemSource::new();

    // Try to get a serif font
    let handle = source.select_best_match(
        &[FamilyName::Serif],
        &Properties::new()
    )?;

    // Get the font data based on the handle type
    let font_data = match handle {
        Handle::Path { path, font_index: _ } => {
            // Load font from file path
            std::fs::read(&path)
                .map_err(|e| format!("Failed to read font file: {}", e))?
        },
        Handle::Memory { bytes, font_index: _ } => {
            // Font is already in memory
            bytes.to_vec()
        },
    };

    // Create a rusttype Font from the font data
    Font::try_from_vec(font_data)
        .ok_or_else(|| "Failed to parse font data".into())
}

fn draw_text(
    image: &mut RgbImage,
    color: Rgb<u8>,
    x: i32,
    y: i32,
    scale: Scale,
    font: &Font<'_>,
    text: &str,
) {
    let v_metrics = font.v_metrics(scale);
    let offset = point(x as f32, y as f32 + v_metrics.ascent);

    // Layout the glyphs
    let glyphs: Vec<_> = font
        .layout(text, scale, offset)
        .collect();

    // Draw each glyph
    for glyph in glyphs {
        if let Some(bounding_box) = glyph.pixel_bounding_box() {
            glyph.draw(|gx, gy, intensity| {
                let px = gx as i32 + bounding_box.min.x;
                let py = gy as i32 + bounding_box.min.y;

                // Check if the pixel is within bounds
                if px >= 0 && px < image.width() as i32 &&
                   py >= 0 && py < image.height() as i32 {
                    // Get the current pixel
                    let pixel = image.get_pixel_mut(px as u32, py as u32);

                    // Blend the color with intensity
                    *pixel = Rgb([
                        ((1.0 - intensity) * pixel[0] as f32 + intensity * color[0] as f32) as u8,
                        ((1.0 - intensity) * pixel[1] as f32 + intensity * color[1] as f32) as u8,
                        ((1.0 - intensity) * pixel[2] as f32 + intensity * color[2] as f32) as u8,
                    ]);
                }
            });
        }
    }
}

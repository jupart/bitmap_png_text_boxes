extern crate clap;
extern crate png;
extern crate bdf;

use std::path::Path;
use std::fs::File;
use std::io::BufWriter;

use png::HasParameters;
use clap::{App, Arg};
use bdf::Font;

fn main() {
    // CLI Application Input
    let matches = App::new("Create text box png's")
                      .version("0.1")
                      .author("jupart <justinwpartain1@gmail.com>")
                      .about("Takes text argument, changes to png")
                      .arg(Arg::with_name("font")
                           .short("f")
                           .long("font")
                           .value_name("FONT")
                           .help("Sets a custom font file")
                           .takes_value(true))
                      .arg(Arg::with_name("INPUT")
                           .help("The input text")
                           .required(true)
                           .index(1))
                      .get_matches();

    let config = match matches.value_of("font") {
        Some(string) => Path::new(string),
        None => Path::new("./scientifica-11.bdf"),
    };

    let input = matches.value_of("INPUT").unwrap();

    // Open the font, get its bounds
    let font = bdf::open(config).unwrap();
    let bounds = font.bounds();
    let max_pixel_width = bounds.width * input.len() as u32;
    let max_pixel_height = bounds.height;

    // Get pixels to paint
    let bits_to_paint = get_glyph_pixels(input, &font);
    let png_pixels = explode_to_png_pixels(bits_to_paint, max_pixel_width, max_pixel_height);

    // Create the png
    write_png(png_pixels, "./test.png", max_pixel_width, max_pixel_height);
}

fn get_glyph_pixels(input_text: &str, font: &Font) -> Vec<u32> {
    let mut bits = Vec::new();
    let font_char_max_x = font.bounds().width as i32;
    let font_char_max_y = font.bounds().height as i32;
    let line_pixel_width = font_char_max_x * input_text.len() as i32;

    for (char_num, character) in input_text.chars().enumerate() {
        let glyph = font.glyphs().get(&character).unwrap();

        let char_left = char_num as i32 * font_char_max_x;
        let char_bottom = glyph.bounds().y;

        for y in 0..glyph.height() {
            for x in 0..glyph.width() {
                let pixel_x = char_left + x as i32;
                let pixel_y = char_bottom + y as i32;

                if glyph.get(x, y) {
                    let png_pixel_num = (pixel_y * line_pixel_width + pixel_x + (font_char_max_y - glyph.bounds().height as i32 - 1) * line_pixel_width) as u32;
                    bits.push(png_pixel_num);
                } else {

                }

            }
        }
    }

    bits
}

fn explode_to_png_pixels(pixels: Vec<u32>, max_x: u32, max_y: u32) -> Vec<u8> {
    let mut png_pixels = Vec::new();

    for i in 0..(max_x * max_y) {
        if pixels.contains(&i) {
            png_pixels.push(235);
            png_pixels.push(219);
            png_pixels.push(178);
            png_pixels.push(255);

        } else {
            png_pixels.push(40);
            png_pixels.push(40);
            png_pixels.push(40);
            png_pixels.push(255);
        }
    }

    png_pixels
}

fn write_png(bits: Vec<u8>, path: &str, max_x: u32, max_y: u32) {
    // println!("Writing {:#?} to png with bounds {}x{}", bits, max_x, max_y);
    let path = Path::new(path);
    let file = File::create(path).unwrap();
    let ref mut w = BufWriter::new(file);

    let mut encoder = png::Encoder::new(w, max_x, max_y);
    encoder.set(png::ColorType::RGBA).set(png::BitDepth::Eight);
    let mut writer = encoder.write_header().unwrap();

    writer.write_image_data(&bits[..]).unwrap();
}

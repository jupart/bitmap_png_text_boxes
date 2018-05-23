#[macro_use]
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
                      .arg(Arg::with_name("padding")
                           .short("p")
                           .long("padding")
                           .value_name("PADDING")
                           .help("Amount of padding in pixels between box and text")
                           .takes_value(true))
                      .arg(Arg::with_name("wrap")
                           .short("w")
                           .long("wrap")
                           .value_name("WRAP")
                           .help("Number of characters to wrap lines at")
                           .takes_value(true))
                      .arg(Arg::with_name("INPUT")
                           .help("The input text")
                           .required(true)
                           .index(1))
                      .get_matches();

    let config = match matches.value_of("font") {
        Some(string) => Path::new(string),
        None => Path::new("./ter-u14n.bdf"),
    };

    let padding = value_t!(matches, "padding", u32).unwrap_or(10);
    let wrap = value_t!(matches, "wrap", u32).unwrap_or(30);
    let input = matches.value_of("INPUT").unwrap();

    // Open the font, get its bounds
    let font = bdf::open(config).unwrap();
    let bounds = font.bounds();

    let input_text_lines = wrap_text(input, wrap);
    let max_line_len = get_max_line_len(&input_text_lines);

    let max_pixel_width = bounds.width * max_line_len + padding * 2;
    let max_pixel_height = bounds.height * input_text_lines.len() as u32 + padding * 2;

    // Get pixels to paint
    let bits_to_paint = get_box_pixels(input_text_lines, &font, padding, max_pixel_width, max_pixel_height);
    let png_pixels = explode_to_png_pixels(bits_to_paint, max_pixel_width, max_pixel_height);

    // Create the png
    write_png(png_pixels, "./test.png", max_pixel_width, max_pixel_height);
}

fn wrap_text(input: &str, wrap_at: u32) -> Vec<String> {
    let mut wrapped_text = Vec::new();
    let mut line_text = String::new();
    let mut num_of_chars = 0;

    for word in input.split_whitespace() {
        num_of_chars = num_of_chars + word.len() + 1;

        if num_of_chars > wrap_at as usize {
            wrapped_text.push(line_text.clone());

            line_text = String::new();
            num_of_chars = word.len() + 1;
        }

        line_text += word;
        line_text += " ";
    }

    if !line_text.is_empty() {
        wrapped_text.push(line_text);
    }

    // println!("Wrapped text at {}: {:?}", wrap_at, wrapped_text);
    wrapped_text
}

fn get_max_line_len(input: &Vec<String>) -> u32 {
    let mut max_len = 0;
    for line in input {
        if line.len() > max_len {
            max_len = line.len();
        }
    }
    // println!("Max len of lines is {}", max_len);
    max_len as u32
}

fn get_box_pixels(input_text: Vec<String>, font: &Font, padding: u32, pixel_w: u32, pixel_h: u32) -> Vec<u32> {
    let mut bits = Vec::new();
    let font_char_max_x = font.bounds().width;
    let font_char_max_y = font.bounds().height;

    for (line_num, line) in input_text.iter().rev().enumerate() {
        for (char_num, character) in line.chars().enumerate() {
            let glyph = font.glyphs().get(&character).unwrap();
            let bitmap = glyph.map();

            let char_left = char_num as u32 * font_char_max_x + padding;
            let char_bottom = (pixel_h as i32 - glyph.bounds().height as i32 - padding as i32 - glyph.bounds().y - line_num as i32 * font_char_max_y as i32) as u32;

            for bit in bitmap.iter() {
                let pixel_x = char_left + bit as u32 % glyph.bounds().width;
                let pixel_y = bit as u32 / glyph.bounds().width + char_bottom - 2;
                let png_pixel_num = (pixel_y * pixel_w + pixel_x) as u32;
                bits.push(png_pixel_num);
            }
        }
    }

    add_box(&mut bits, pixel_w, pixel_h);

    bits
}

fn add_box(character_pixels: &mut Vec<u32>, box_length: u32, box_height: u32) {
    // Top and bottom line
    for i in 0..box_length {
        character_pixels.push(i);
        character_pixels.push(i + box_length);
        character_pixels.push(i + (box_height - 1) * box_length);
        character_pixels.push(i + (box_height - 2) * box_length);
    }

    // Sides
    for i in 0..box_height {
        character_pixels.push(i * box_length);
        character_pixels.push(i * box_length + 1);
        character_pixels.push(((i + 1) * box_length) - 1);
        character_pixels.push(((i + 1) * box_length) - 2);
    }
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

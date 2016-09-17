#[macro_use]
extern crate clap;

extern crate image;

use clap::App;
use image::DynamicImage;
use std::io::{Error, ErrorKind, Result};

const DEFAULT_FACTOR: u32 = 4;
const CHANNELS: usize = 3;

type RgbImage = Vec<u8>;
type RgbColor = [u8; CHANNELS];

fn main() {
    let matches = App::new("PNG Scale")
        .args_from_usage("-i <input.png> 'Filename to scale'
                          -o <scaled.png> 'Output filename'
                          -f <number> 'Scaling factor'")
        .get_matches();

    let input_filename = value_t_or_exit!(matches, "i", String);
    let output_filename = value_t_or_exit!(matches, "o", String);
    let factor = if let Ok(factor) = value_t!(matches, "f", u32) {
        factor
    } else {
        DEFAULT_FACTOR
    };

    match do_checked_main(input_filename, output_filename, factor) {
        Ok(_) => println!("OK"),
        Err(e) => println!("Error: {:?}", e),
    }
}

fn do_checked_main(input_filename: String, output_filename: String, factor: u32) -> Result<()> {
    let factor = factor as usize;
    let data = image::open(input_filename).unwrap();
    match data {
        DynamicImage::ImageRgb8(rgb_image) => {
            let (width, height) = rgb_image.dimensions();
            let width = width as usize;
            let height = height as usize;

            let input_image = rgb_image.into_vec();
            let (output_image, scaled_width, scaled_height) =
                try!(scale_nearest_neighbor(input_image, width, height, factor));
            try!(image::save_buffer(output_filename,
                                    &output_image[..],
                                    scaled_width as u32,
                                    scaled_height as u32,
                                    image::ColorType::RGB(8)));
        }
        _ => {
            let e = Error::new(ErrorKind::InvalidInput,
                               "Error: unsupported image. Only RGB images are supported.");
            return Err(e);
        }
    }

    Ok(())
}

fn scale_nearest_neighbor(image: RgbImage,
                          width: usize,
                          height: usize,
                          factor: usize)
                          -> Result<(RgbImage, usize, usize)> {
    let scaled_width = width * factor;
    let scaled_height = height * factor;
    let mut output = vec![0; scaled_width * scaled_height * CHANNELS];
    for x in 0..width {
        for y in 0..height {
            let color = get_pixel(x, y, &image, width, height);
            for x_offset in 0..factor {
                for y_offset in 0..factor {
                    put_pixel(x * factor + x_offset,
                              y * factor + y_offset,
                              color,
                              &mut output,
                              scaled_width,
                              scaled_height);
                }
            }
        }
    }

    Ok((output, scaled_width, scaled_height))
}

fn put_pixel(x: usize,
             y: usize,
             color: RgbColor,
             image: &mut RgbImage,
             width: usize,
             height: usize) {
    let index = (x + y * height) * CHANNELS;
    for i in 0..CHANNELS {
        image[index + i] = color[i];
    }
}

fn get_pixel(x: usize, y: usize, image: &RgbImage, width: usize, height: usize) -> RgbColor {
    let mut color = [0; CHANNELS];
    let index = (x + y * height) * CHANNELS;
    for i in 0..CHANNELS {
        color[i] = image[index + i];
    }
    color
}

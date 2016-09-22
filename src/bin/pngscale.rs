#[macro_use]
extern crate clap;

extern crate image;

use clap::App;
use image::DynamicImage;
use std::io::{Error, ErrorKind, Result};

const DEFAULT_FACTOR: u32 = 4;
const CHANNELS: usize = 3;

pub type RgbColor = [u8; CHANNELS];

struct RgbImage {
    buffer: Vec<u8>,
    width: usize,
    height: usize,
}

impl RgbImage {
    pub fn new(width: usize, height: usize) -> Self {
        RgbImage {
            buffer: vec![0; width * height * CHANNELS],
            width: width,
            height: height,
        }
    }

    pub fn with_buffer(buffer: Vec<u8>, width: usize, height: usize) -> Self {
        RgbImage {
            buffer: buffer,
            width: width,
            height: height,
        }
    }

    pub fn put_pixel(&mut self, x: usize, y: usize, color: RgbColor) {
        let index = (x + y * self.width) * CHANNELS;
        for i in 0..CHANNELS {
            self.buffer[index + i] = color[i];
        }
    }

    pub fn get_pixel(&self, x: usize, y: usize) -> RgbColor {
        let mut color = [0; CHANNELS];
        let index = (x + y * self.width) * CHANNELS;
        for i in 0..CHANNELS {
            color[i] = self.buffer[index + i];
        }
        color
    }

    pub fn scale_nearest_neighbor(&self, factor: usize) -> Result<RgbImage> {
        let mut output = RgbImage::new(self.width * factor, self.height * factor);

        for y in 0..self.height {
            for x in 0..self.width {
                let color = self.get_pixel(x, y);
                for x_offset in 0..factor {
                    for y_offset in 0..factor {
                        let out_x = x * factor + x_offset;
                        let out_y = y * factor + y_offset;
                        output.put_pixel(out_x, out_y, color);
                    }
                }
            }
        }

        Ok(output)
    }

    pub fn scale_bilinear(&self, factor: usize) -> Result<RgbImage> {
        let mut output = RgbImage::new(self.width * factor, self.height * factor);

        for y0 in 0..(self.height - 1) {
            for x0 in 0..(self.width - 1) {
                let q11 = self.get_pixel(x0, y0);
                let q21 = self.get_pixel(x0 + 1, y0);
                let q12 = self.get_pixel(x0, y0 + 1);
                let q22 = self.get_pixel(x0 + 1, y0 + 1);
                let x1 = x0 * factor;
                let y1 = y0 * factor;
                let x2 = (x0 + 1) * factor;
                let y2 = (y0 + 1) * factor;

                for y in y1..y2 {
                    for x in x1..x2 {
                        let x = x as u64;
                        let y = y as u64;
                        let x1 = x1 as u64;
                        let y1 = y1 as u64;
                        let x2 = x2 as u64;
                        let y2 = y2 as u64;

                        let area11 = (x - x1) * (y - y1);
                        let area21 = (x2 - x) * (y - y1);
                        let area12 = (x - x1) * (y2 - y);
                        let area22 = (x2 - x) * (y2 - y);
                        let area = (x2 - x1) * (y2 - y1);

                        let mut q: RgbColor = [0; CHANNELS];
                        for i in 0..CHANNELS {
                            // swaped colors
                            let q11_channel = q22[i] as u64;
                            let q21_channel = q12[i] as u64;
                            let q12_channel = q21[i] as u64;
                            let q22_channel = q11[i] as u64;
                            let q_channel = (q11_channel * area11 + q21_channel * area21 +
                                             q12_channel * area12 +
                                             q22_channel * area22) /
                                            area;
                            q[i] = q_channel as u8;
                        }

                        let x = x as usize;
                        let y = y as usize;
                        output.put_pixel(x, y, q);
                    }
                }
            }
        }

        Ok(output)
    }
}

fn main() {
    let matches = App::new("PNG Scale")
        .args_from_usage("-i <input.png> 'Filename to scale'
                          -o <scaled.png> 'Output filename'
                          -f <number> 'Scaling factor'
                          -a <nearest|bilinear> 'Scaling algorithm'")
        .get_matches();

    let algorithm = value_t_or_exit!(matches, "a", String);
    let input_filename = value_t_or_exit!(matches, "i", String);
    let output_filename = value_t_or_exit!(matches, "o", String);
    let factor = if let Ok(factor) = value_t!(matches, "f", u32) {
        factor
    } else {
        DEFAULT_FACTOR
    };

    match do_checked_main(input_filename, output_filename, factor, algorithm.as_str()) {
        Ok(_) => println!("OK"),
        Err(e) => println!("Error: {:?}", e),
    }
}

fn do_checked_main(input_filename: String,
                   output_filename: String,
                   factor: u32,
                   algorithm: &str)
                   -> Result<()> {
    let data = match image::open(input_filename) {
        Ok(data) => data,
        Err(e) => {
            let e = Error::new(ErrorKind::Other, format!("{:?}", e));
            return Err(e);
        }
    };

    match data {
        DynamicImage::ImageRgb8(image_data) => {
            let (width, height) = image_data.dimensions();
            let width = width as usize;
            let height = height as usize;
            let factor = factor as usize;
            let input_image = RgbImage::with_buffer(image_data.into_vec(), width, height);
            let output_image = match algorithm {
                "nearest" => try!(input_image.scale_nearest_neighbor(factor)),
                "bilinear" => try!(input_image.scale_bilinear(factor)),
                _ => unreachable!(),
            };
            image::save_buffer(output_filename,
                               output_image.buffer.as_slice(),
                               output_image.width as u32,
                               output_image.height as u32,
                               image::ColorType::RGB(8))
        }
        _ => {
            let e = Error::new(ErrorKind::InvalidInput,
                               "Error: unsupported image. Only RGB images are supported.");
            Err(e)
        }
    }
}

#[macro_use]
extern crate clap;

extern crate image;

extern crate nalgebra;

extern crate rust_experiments;

use clap::App;
use image::DynamicImage;
use nalgebra::{Dot, Vector3 as Vec3, Vector4 as Vec4};
use rust_experiments::encoding::bitreader::BitReader;
use rust_experiments::encoding::bitwriter::BitWriter;
use rust_experiments::encoding::huffman::{HuffmanEncoder, HuffmanDecoder};
use std::fs::File;
use std::io::{Cursor, Error, ErrorKind, Result};
use std::mem;

const DEFAULT_FACTOR: u32 = 4;

const RGB_CHANNELS: usize = 3;
const YCBCR_CHANNELS: usize = 3;

const CHAR_LENGTH: usize = 4;

type RgbColor = [u8; RGB_CHANNELS];

struct RgbImage {
    buffer: Vec<u8>,
    width: usize,
    height: usize,
}

struct YCbCr {
    y: f64,
    cb: f64,
    cr: f64,
}

const FLOAT_OFFSET: f64 = 1e-6;
const FIXED_OFFSET: u64 = 1_000;

trait ToFixed {
    fn to_fixed(&self) -> u32;
}

trait FromFixed {
    fn fixed_to_f64(&self) -> f64;
}

impl ToFixed for f64 {
    fn to_fixed(&self) -> u32 {
        let result = (self / FLOAT_OFFSET) as u64;
        let result = result / FIXED_OFFSET;
        result as u32
    }
}

impl FromFixed for u32 {
    fn fixed_to_f64(&self) -> f64 {
        let result = *self as u64 * FIXED_OFFSET;
        result as f64 * FLOAT_OFFSET
    }
}

impl RgbImage {
    pub fn new(width: usize, height: usize) -> Self {
        RgbImage {
            buffer: vec![0; width * height * RGB_CHANNELS],
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
        let index = (x + y * self.width) * RGB_CHANNELS;
        for i in 0..RGB_CHANNELS {
            self.buffer[index + i] = color[i];
        }
    }

    pub fn get_pixel(&self, x: usize, y: usize) -> RgbColor {
        let mut color = [0; RGB_CHANNELS];
        let index = (x + y * self.width) * RGB_CHANNELS;
        for i in 0..RGB_CHANNELS {
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

        let bucket_width = output.width / (self.width - 1);
        let bucket_height = output.height / (self.height - 1);

        let compute_color = |x, y, x1, y1, x2, y2, qs: &[RgbColor]| {
            let full_area = (x2 - x1) * (y2 - y1);

            let area11 = (x - x1) * (y - y1);
            let area21 = (x2 - x) * (y - y1);
            let area12 = (x - x1) * (y2 - y);
            let area22 = (x2 - x) * (y2 - y);
            let areas = Vec4::new(area11, area21, area12, area22);

            let mut new_color: RgbColor = [0; RGB_CHANNELS];
            for channel in 0..RGB_CHANNELS {
                // swaped colors, see https://tinyurl.com/hq3dedl
                let q_channel = Vec4::new(qs[3][channel] as usize,
                                          qs[2][channel] as usize,
                                          qs[1][channel] as usize,
                                          qs[0][channel] as usize);
                let new_channel = q_channel.dot(&areas) / full_area;
                new_color[channel] = new_channel as u8;
            }

            new_color
        };

        let interpolate_bucket = |output: &mut RgbImage, x1, y1, x2, y2, qs: &[RgbColor]| {
            for y in y1..y2 {
                for x in x1..x2 {
                    let new_color = compute_color(x, y, x1, y1, x2, y2, qs);
                    output.put_pixel(x, y, new_color);
                }
            }
        };

        for y0 in 0..(self.height - 1) {
            for x0 in 0..(self.width - 1) {
                let q11 = self.get_pixel(x0, y0);
                let q21 = self.get_pixel(x0 + 1, y0);
                let q12 = self.get_pixel(x0, y0 + 1);
                let q22 = self.get_pixel(x0 + 1, y0 + 1);
                let qs = [q11, q21, q12, q22];
                let x1 = x0 * bucket_width;
                let y1 = y0 * bucket_height;
                let x2 = (x0 + 1) * bucket_width;
                let y2 = (y0 + 1) * bucket_height;
                interpolate_bucket(&mut output, x1, y1, x2, y2, &qs);

                let is_right_edge = x0 == self.width - 2;
                if is_right_edge {
                    let qs = [q21, q22, q22, q22];
                    let x1 = x2;
                    let x2 = output.width;
                    interpolate_bucket(&mut output, x1, y1, x2, y2, &qs);
                }

                let is_bottom_edge = y0 == self.height - 2;
                if is_bottom_edge {
                    let qs = [q12, q22, q12, q22];
                    let y1 = y2;
                    let y2 = output.height;
                    interpolate_bucket(&mut output, x1, y1, x2, y2, &qs);
                }

                if is_right_edge && is_bottom_edge {
                    let q22 = self.get_pixel(self.width - 1, self.height - 1);
                    let qs = [q22, q22, q22, q22];
                    let x1 = x2;
                    let y1 = y2;
                    let x2 = output.width;
                    let y2 = output.height;
                    interpolate_bucket(&mut output, x1, y1, x2, y2, &qs);
                }
            }
        }

        Ok(output)
    }

    pub fn grayscale(&self) -> Result<RgbImage> {
        let mut output = RgbImage::new(self.width, self.height);

        for y in 0..output.height {
            for x in 0..output.width {
                let color = self.get_pixel(x, y);
                let luma = RgbImage::to_ycbcr(&color).y as u8;
                let gray_color = [luma; RGB_CHANNELS];
                output.put_pixel(x, y, gray_color);
            }
        }

        Ok(output)
    }

    pub fn compress(&self, output_filename: &str) -> Result<()> {
        let n = self.width * self.height;

        let mut y_vec = Vec::with_capacity(n);
        let mut cb_vec = Vec::with_capacity(n);
        let mut cr_vec = Vec::with_capacity(n);
        for y in 0..self.height {
            for x in 0..self.width {
                let color = self.get_pixel(x, y);
                y_vec.push(RgbImage::to_ycbcr(&color).y.to_fixed());
                cb_vec.push(RgbImage::to_ycbcr(&color).cb.to_fixed());
                cr_vec.push(RgbImage::to_ycbcr(&color).cr.to_fixed());
            }
        }

        let mut writer = BitWriter::new(Vec::with_capacity(n * YCBCR_CHANNELS));

        for &y in &y_vec[..] {
            try!(writer.write_u32(y));
        }

        for &cb in &cb_vec[..] {
            try!(writer.write_u32(cb));
        }

        for &cr in &cr_vec[..] {
            try!(writer.write_u32(cr));
        }

        try!(writer.flush());

        let width = self.width as u64;
        let height = self.height as u64;
        let f = try!(File::create(output_filename));
        let mut header_writer = BitWriter::new(f);
        try!(header_writer.write_u64(width));
        try!(header_writer.write_u64(height));

        let mut encoder = HuffmanEncoder::new(header_writer.get_mut(), CHAR_LENGTH);
        let data = writer.get_ref().as_slice();
        let reader = Cursor::new(data);
        try!(encoder.analyze(reader.clone()));
        try!(encoder.analyze_finish());
        try!(encoder.compress(reader));
        try!(encoder.compress_finish());

        Ok(())
    }

    pub fn decompress(input_filename: &str, output_filename: &str) -> Result<()> {
        let f = try!(File::open(input_filename));
        let mut reader = BitReader::new(f);
        let width = try!(reader.read_u64()) as usize;
        let height = try!(reader.read_u64()) as usize;
        let n = width * height;
        let pixels_length = (n * YCBCR_CHANNELS) * mem::size_of::<u32>();
        let pixels_length_bits = pixels_length as u64 * 8;
        let mut pixels_writer = Vec::with_capacity(pixels_length);

        let mut coder = try!(HuffmanDecoder::new(reader.get_ref()));
        let data_offset_bit = coder.data_offset_bit();
        try!(coder.decode(&mut pixels_writer, data_offset_bit, pixels_length_bits));
        let mut reader = BitReader::new(pixels_writer.as_slice());

        let mut y_vec = Vec::with_capacity(n);
        let mut cb_vec = Vec::with_capacity(n);
        let mut cr_vec = Vec::with_capacity(n);

        for _ in 0..n {
            let value = try!(reader.read_u32()).fixed_to_f64();
            y_vec.push(value);
        }

        for _ in 0..n {
            let value = try!(reader.read_u32()).fixed_to_f64();
            cb_vec.push(value);
        }

        for _ in 0..n {
            let value = try!(reader.read_u32()).fixed_to_f64();
            cr_vec.push(value);
        }

        // pixels to rgb
        let mut output = RgbImage::new(width, height);
        let mut i = 0;
        for y in 0..height {
            for x in 0..width {
                let compressed = YCbCr {
                    y: y_vec[i],
                    cb: cb_vec[i],
                    cr: cr_vec[i],
                };
                let new_pixel = RgbImage::to_rgb(&compressed);
                output.put_pixel(x, y, new_pixel);
                i += 1;
            }
        }

        try!(image::save_buffer(output_filename,
                                output.buffer.as_slice(),
                                output.width as u32,
                                output.height as u32,
                                image::ColorType::RGB(8)));

        Ok(())
    }

    // see https://en.wikipedia.org/wiki/YCbCr#JPEG_conversion

    fn to_ycbcr(color: &RgbColor) -> YCbCr {
        let offset = Vec3::new(0.0, 128.0, 128.0);
        let y_factors = Vec3::new(0.299, 0.587, 0.114);
        let cb_factors = Vec3::new(-0.168736, -0.331264, 0.5);
        let cr_factors = Vec3::new(0.5, -0.418688, -0.081312);

        let rgb = Vec3::new(color[0] as f64, color[1] as f64, color[2] as f64);
        let result = Vec3::new(y_factors.dot(&rgb),
                               cb_factors.dot(&rgb),
                               cr_factors.dot(&rgb));
        let result = result + offset;

        YCbCr {
            y: result.x,
            cb: result.y,
            cr: result.z,
        }
    }

    fn to_rgb(color: &YCbCr) -> RgbColor {
        let aligned_cr = color.cr - 128.0;
        let aligned_cb = color.cb - 128.0;
        let r = color.y + 1.402 * aligned_cr;
        let g = color.y - 0.344136 * aligned_cb - 0.714136 * aligned_cr;
        let b = color.y + 1.772 * aligned_cb;
        [r as u8, g as u8, b as u8]
    }
}

fn main() {
    let usage = "-i <input.png> 'Filename to convert'
                -o <output.png> 'Output filename'
                [-f <number>] 'Scaling factor'
                -a <nearest|bilinear|grayscale|compress|decompress> 'Conversion algorithm'";
    let matches = App::new("PNG Convert")
        .args_from_usage(usage)
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
    // FIXME: refactor
    if algorithm == "decompress" {
        return RgbImage::decompress(input_filename.as_str(), output_filename.as_str());
    } else {
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
                match algorithm {
                    "compress" => input_image.compress(output_filename.as_str()),
                    _ => {
                        let output_image = match algorithm {
                            "nearest" => try!(input_image.scale_nearest_neighbor(factor)),
                            "bilinear" => try!(input_image.scale_bilinear(factor)),
                            "grayscale" => try!(input_image.grayscale()),
                            _ => unreachable!(),
                        };
                        image::save_buffer(output_filename,
                                           output_image.buffer.as_slice(),
                                           output_image.width as u32,
                                           output_image.height as u32,
                                           image::ColorType::RGB(8))
                    }
                }
            }
            _ => {
                let e = Error::new(ErrorKind::InvalidInput,
                                   "Error: unsupported image. Only RGB images are supported.");
                Err(e)
            }
        }
    }
}

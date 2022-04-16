use image::ImageError;
use image::{
    imageops::FilterType::Triangle, io::Reader, DynamicImage, GenericImageView, ImageFormat,
};
use std::convert::TryInto;
use std::env::{args, Args};
use std::fs::File;
use std::io::BufReader;

#[derive(Debug)]
enum ImageDataErrors {
    DifferentImageFormat,
    BufferTooSmall,
    ImageBufferSaveFailed(ImageError),
}

#[derive(Debug)]
struct FloatingImage {
    width: u32,
    height: u32,
    data: Vec<u8>,
    name: String,
}

impl FloatingImage {
    fn new(w: u32, h: u32, name: String) -> Self {
        let buffer_capacity: u32 = h * w * 4;
        let buffer: Vec<u8> = Vec::with_capacity(buffer_capacity.try_into().unwrap());

        return FloatingImage {
            width: w,
            height: h,
            data: buffer,
            name: name,
        };
    }

    fn set_data(&mut self, data: Vec<u8>) -> Result<(), ImageDataErrors> {
        if data.len() > self.data.capacity() {
            return Err(ImageDataErrors::BufferTooSmall);
        }

        self.data = data;
        return Ok(());
    }
}

fn main() -> Result<(), ImageDataErrors> {
    let mut a: Args = args();
    let first_path = a.nth(1).unwrap();
    let second_path = a.nth(0).unwrap();
    let third_path = a.nth(0).unwrap();

    let (raw_im1, im1_format) = find_image(first_path);
    let (raw_im2, im2_format) = find_image(second_path);

    if im1_format != im2_format {
        return Err(ImageDataErrors::DifferentImageFormat);
    }

    let (im1, im2) = standardize_size(raw_im1, raw_im2);
    let mut im_output = FloatingImage::new(im1.width(), im1.height(), third_path);

    let combined_data = combine_images(im1, im2);

    im_output.set_data(combined_data)?;

    if let Err(e) = image::save_buffer_with_format(
        im_output.name,
        &im_output.data,
        im_output.width,
        im_output.height,
        image::ColorType::Rgba8,
        im1_format,
    ) {
        return Err(ImageDataErrors::ImageBufferSaveFailed(e));
    }

    // println!("{:?}", im_output);
    return Ok(());
}

fn find_image(filepath: String) -> (DynamicImage, ImageFormat) {
    let image_reader: Reader<BufReader<File>> = Reader::open(filepath).unwrap();
    let image_format = image_reader.format().unwrap();
    let image: DynamicImage = image_reader.decode().unwrap();

    return (image, image_format);
}

fn get_smallest_dimension(dim1: (u32, u32), dim2: (u32, u32)) -> (u32, u32) {
    let pixel1 = dim1.0 * dim1.1;
    let pixel2 = dim2.0 * dim2.1;
    return if pixel1 < pixel2 { dim1 } else { dim2 };
}

fn standardize_size(im1: DynamicImage, im2: DynamicImage) -> (DynamicImage, DynamicImage) {
    let (w, h) = get_smallest_dimension(im1.dimensions(), im2.dimensions());
    println!("width: {}, height: {}\n", w, h);

    if im2.dimensions() == (w, h) {
        return (im1.resize_exact(w, h, Triangle), im2);
    }

    return (im1, im2.resize_exact(w, h, Triangle));
}

fn combine_images(im1: DynamicImage, im2: DynamicImage) -> Vec<u8> {
    let vec1 = im1.to_rgba8().into_vec();
    let vec2 = im2.to_rgba8().into_vec();

    return alternate_pixels(vec1, vec2);
}

fn alternate_pixels(v1: Vec<u8>, v2: Vec<u8>) -> Vec<u8> {
    let mut combined = vec![0u8; v1.len()];

    let mut i = 0;
    while i < v1.len() {
        if i % 8 == 0 {
            combined.splice(i..=i + 3, set_rgba(&v1, i, i + 3));
        } else {
            combined.splice(i..=i + 3, set_rgba(&v2, i, i + 3));
        }
        i += 4
    }

    return combined;
}

fn set_rgba(v: &Vec<u8>, start: usize, end: usize) -> Vec<u8> {
    let mut rgba: Vec<u8> = Vec::new();
    for i in start..=end {
        let val: u8 = match v.get(i) {
            Some(d) => *d,
            None => panic!("Index out of bounds"),
        };

        rgba.push(val);
    }

    return rgba;
}

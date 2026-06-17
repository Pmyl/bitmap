use std::{
    collections::{HashMap, HashSet},
    error::Error,
    io::Write,
};

pub struct Bitmap {
    file_header: FileHeader,
    dib_header: DibHeader,
    color_table: Vec<u32>,
    pixel_array: Vec<u32>,
}

impl Bitmap {
    pub fn write(&self, writer: &mut impl Write) -> Result<(), Box<dyn Error>> {
        writer.write(as_bytes(&self.file_header))?;
        writer.write(as_bytes(&self.dib_header))?;
        for byte in &self.color_table {
            writer.write(&byte.to_le_bytes())?;
        }
        for byte in &self.pixel_array {
            writer.write(&byte.to_be_bytes())?;
        }

        Ok(())
    }
}

#[repr(packed)]
pub struct FileHeader {
    pub identifier: [u8; 2],
    pub size: u32,
    pub reserved_1: u16,
    pub reserved_2: u16,
    pub offset: u32,
}

#[repr(packed)]
pub struct DibHeader {
    pub size_of_this_header: u32,
    pub width_in_pixels: i32,
    pub height_in_pixels: i32,
    pub color_planes: u16,
    pub bits_per_pixel: u16,
    pub compression_method: u32,
    pub image_size: u32,
    pub horizontal_resolution: i32,
    pub vertical_resolution: i32,
    pub number_of_colors: u32,
    pub number_of_important_colors: u32,
}

pub struct Pixel(Point, u32);

pub struct BitmapMaker {
    pub width: usize,
    pub height: usize,
    pub pixels: Vec<Pixel>,
}

#[derive(Hash, PartialEq, Eq, Clone, Copy)]
pub struct Point {
    pub x: usize,
    pub y: usize,
}

impl BitmapMaker {
    pub fn new(width: usize, height: usize) -> Self {
        BitmapMaker {
            width,
            height,
            pixels: vec![],
        }
    }

    pub fn with(mut self, point: Point, colour: u32) -> Self {
        self.pixels.push(Pixel(point, colour));
        return self;
    }

    pub fn make(self) -> Result<Bitmap, Box<dyn Error>> {
        let empty_colour = 0xFFFFFF;

        let mut unique_colours = self
            .pixels
            .iter()
            .map(|x| x.1)
            .collect::<HashSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();

        unique_colours.push(empty_colour);

        let bits_per_pixel: u16 = unique_colours.len().next_power_of_two().ilog2() as u16;
        // TODO handle bits per pixels when they are not divisor of 32 (ex 3)
        let bits_per_pixel = bits_per_pixel.next_power_of_two();

        let width_in_pixels: i32 = self.width as i32;
        let height_in_pixels: i32 = self.height as i32;

        let row_size =
            ((bits_per_pixel as f32 * width_in_pixels as f32 / 32.0).ceil() * 4.0) as u32;
        let pixel_array_size = row_size * height_in_pixels.unsigned_abs();

        let file_header_size: u32 = 14;
        let dib_header_size: u32 = 40;
        let color_table_in_bytes: u32 = 2u32.pow(bits_per_pixel as u32) * 4;

        let mut color_table = vec![empty_colour; color_table_in_bytes as usize / 4];

        for (i, uc) in unique_colours.into_iter().enumerate() {
            color_table[i] = uc;
        }

        let mut pixel_array = Vec::with_capacity((row_size / 4 * pixel_array_size) as usize);

        let map = self
            .pixels
            .iter()
            .map(|x| (x.0, x.1))
            .collect::<HashMap<_, _>>();

        let mut row_part: u32 = 0;
        let mut pixel_added = 0;
        for row in (0..height_in_pixels as usize).rev() {
            for col in 0..width_in_pixels as usize {
                let colour = map.get(&Point { x: col, y: row }).unwrap_or(&empty_colour);
                let colour_index = color_table.iter().position(|c| c == colour).unwrap() as u32;

                //
                // Is 32 or 31?
                let shift = 32 - bits_per_pixel * (pixel_added + 1);
                let shifted_colour_index = colour_index << shift;
                row_part = row_part | shifted_colour_index;
                pixel_added = pixel_added + 1;

                if pixel_added * bits_per_pixel == 32 {
                    pixel_array.push(row_part);
                    pixel_added = 0;
                    row_part = 0;
                }
            }

            if pixel_added != 0 {
                pixel_array.push(row_part);
                pixel_added = 0;
                row_part = 0;
            }
        }

        Ok(Bitmap {
            file_header: FileHeader {
                identifier: [b'B', b'M'],
                size: file_header_size + dib_header_size + color_table_in_bytes + pixel_array_size,
                reserved_1: 0,
                reserved_2: 0,
                offset: file_header_size + dib_header_size + color_table_in_bytes,
            },
            dib_header: DibHeader {
                size_of_this_header: dib_header_size,
                width_in_pixels: width_in_pixels,
                height_in_pixels: height_in_pixels,
                color_planes: 1,
                bits_per_pixel: bits_per_pixel,
                compression_method: 0,
                image_size: pixel_array_size,
                horizontal_resolution: 2835, // Print resolution of the image, 72 DPI × 39.3701 inches per metre yields 2834.6472
                vertical_resolution: 2835, // Print resolution of the image, 72 DPI × 39.3701 inches per metre yields 2834.6472
                number_of_colors: 0,       // 0 means that the number of colors is 2^bits_per_pixel
                number_of_important_colors: 0,
            },
            color_table: color_table, // Red and Blue
            pixel_array: pixel_array,
        })
    }
}

// Assume it's going to run on a machine that uses Little Endian
// If not, don't call me
fn as_bytes<T>(value: &T) -> &[u8] {
    unsafe {
        std::slice::from_raw_parts(
            std::ptr::from_ref(value).cast::<u8>(),
            std::mem::size_of::<T>(),
        )
    }
}

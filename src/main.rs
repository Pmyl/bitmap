use std::{error::Error, fs, io::Write};

struct Bitmap {
    file_header: FileHeader,
    dib_header: DibHeader,
    color_table: Vec<u32>,
    pixel_array: Vec<u32>,
}

#[repr(packed)]
struct FileHeader {
    identifier: [u8; 2],
    size: u32,
    reserved_1: u16,
    reserved_2: u16,
    offset: u32,
}

#[repr(packed)]
struct DibHeader {
    size_of_this_header: u32,
    width_in_pixels: i32,
    height_in_pixels: i32,
    color_planes: u16,
    bits_per_pixel: u16,
    compression_method: u32,
    image_size: u32,
    horizontal_resolution: i32,
    vertical_resolution: i32,
    number_of_colors: u32,
    number_of_important_colors: u32,
}

struct Config {
    width: u32,
    height: u32,
    checker_size: u32,
}

fn main() -> Result<(), Box<dyn Error>> {
    let config = read_config();

    let bits_per_pixel: u16 = 1;
    let width_in_pixels: i32 = config.width as i32;
    let height_in_pixels: i32 = config.height as i32;

    let row_size = ((bits_per_pixel as f32 * width_in_pixels as f32 / 32.0).ceil() * 4.0) as u32;
    let pixel_array_size = row_size * height_in_pixels.unsigned_abs();

    let file_header_size: u32 = 14;
    let dib_header_size: u32 = 40;
    let color_table_size: u32 = 2u32.pow(bits_per_pixel as u32) * 4;

    let mut color_table = Vec::with_capacity(color_table_size as usize);
    color_table.push(0xFFAE0000);
    color_table.push(0xFF0000AE);

    let mut pixel_array = Vec::with_capacity((row_size / 4 * pixel_array_size) as usize);
    // 10101000 00000000 00000000 00000000
    // BRBRBPPP PPPPPPPP PPPPPPPP PPPPPPPP

    let square_size_in_pixels = config
        .width
        .checked_div(config.checker_size)
        .ok_or_else(|| "Invalid checker size")? as i32;

    for y in 0..height_in_pixels {
        let mut remaining_width = width_in_pixels as u32;

        let mut x: i32 = 0;

        while remaining_width > 0 {
            let mut row_part = 0;
            let width_of_row_part = remaining_width.min(32);
            remaining_width = remaining_width.saturating_sub(32);

            for i in 0..width_of_row_part {
                let x_square = x
                    .checked_div(square_size_in_pixels)
                    .ok_or_else(|| "Invalid checker size")?;
                let y_square = y
                    .checked_div(square_size_in_pixels)
                    .ok_or_else(|| "Invalid checker size")?;

                if (x_square + y_square) % 2 == 0 {
                    row_part += 1 << (31 - i);
                }

                x += 1;
            }
            pixel_array.push(row_part);
        }
    }

    let bitmap = Bitmap {
        file_header: FileHeader {
            identifier: [b'B', b'M'],
            size: file_header_size + dib_header_size + color_table_size + pixel_array_size,
            reserved_1: 0,
            reserved_2: 0,
            offset: file_header_size + dib_header_size + color_table_size,
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
    };

    let mut image_file = fs::OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open("image.bmp")
        .expect("Should be able to open a file");

    image_file.write(as_bytes(&bitmap.file_header))?;
    image_file.write(as_bytes(&bitmap.dib_header))?;
    for byte in bitmap.color_table {
        image_file.write(&byte.to_le_bytes())?;
    }
    for byte in bitmap.pixel_array {
        image_file.write(&byte.to_be_bytes())?;
    }

    Ok(())
}

fn read_config() -> Config {
    let mut args = std::env::args()
        .skip(1)
        .collect::<Vec<String>>()
        .into_iter();
    let width = args
        .next()
        .unwrap_or("1".into())
        .parse::<u32>()
        .unwrap_or(1);
    let height = args
        .next()
        .unwrap_or("1".into())
        .parse::<u32>()
        .unwrap_or(1);
    let checker_size = args
        .next()
        .unwrap_or("1".into())
        .parse::<u32>()
        .unwrap_or(1);

    Config {
        width,
        height,
        checker_size,
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

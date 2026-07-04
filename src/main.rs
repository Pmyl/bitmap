use std::{error::Error, fs};

use bitmap::{BitmapMaker, Point};

struct Config {
    width: u32,
    height: u32,
}

fn main() -> Result<(), Box<dyn Error>> {
    let config = read_config();

    let bitmap = BitmapMaker::new(config.width as usize, config.height as usize)
        .with(Point { x: 0, y: 0 }, 0xFFFFFF00)
        .with(Point { x: 0, y: 1 }, 0xFFFF00FF)
        .with(Point { x: 0, y: 2 }, 0xFF00FFFF)
        .with(Point { x: 0, y: 3 }, 0xFF0000FF)
        .with(Point { x: 0, y: 4 }, 0xFF000AFF)
        .make()?;

    let mut image_file = fs::OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open("image.bmp")
        .expect("Should be able to open a file");

    bitmap.write(&mut image_file)?;

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

    Config { width, height }
}

use image::{ImageReader, ImageFormat};
use std::error::Error;

pub fn ppm_to_jpeg(ppm_path: &str, jpeg_path: &str) -> Result<(), Box<dyn Error>> {
    let img = ImageReader::open(ppm_path)?.with_guessed_format()?.decode()?;
    img.save_with_format(jpeg_path, ImageFormat::Jpeg)?;
    Ok(())
}
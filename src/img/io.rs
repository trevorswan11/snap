use crate::img::image::*;
use crate::img::matrix::*;

use image::{ImageFormat, ImageReader, load_from_memory};
use std::error::Error;
use std::fs::File;
use std::io::{self, BufRead, BufWriter, Cursor, Read, Write};
use std::path::Path;

/// Represents the two common types of PPM files
#[derive(Debug, Clone)]
pub enum PPMFormat {
    P3,
    P6,
}

/// Infers the image type from a given file path and maps it to ImageFormat
pub fn infer_type<P: AsRef<Path>>(path: P) -> Result<ImageFormat, Box<dyn Error>> {
    let ext = path
        .as_ref()
        .extension()
        .and_then(|e| e.to_str())
        .ok_or("Missing or invalid file extension")?
        .to_lowercase();

    match ext.as_str() {
        "png" => Ok(ImageFormat::Png),
        "jpg" | "jpeg" => Ok(ImageFormat::Jpeg),
        "gif" => Ok(ImageFormat::Gif),
        "webp" => Ok(ImageFormat::WebP),
        "pnm" => Ok(ImageFormat::Pnm),
        "tiff" | "tif" => Ok(ImageFormat::Tiff),
        "tga" => Ok(ImageFormat::Tga),
        "dds" => Ok(ImageFormat::Dds),
        "bmp" => Ok(ImageFormat::Bmp),
        "ico" => Ok(ImageFormat::Ico),
        "hdr" => Ok(ImageFormat::Hdr),
        "exr" => Ok(ImageFormat::OpenExr),
        "ff" | "farbfeld" => Ok(ImageFormat::Farbfeld),
        "avif" => Ok(ImageFormat::Avif),
        "qoi" => Ok(ImageFormat::Qoi),
        "pcx" => Ok(ImageFormat::Pcx),
        _ => Err("Unknown or unsupported image file extension".into()),
    }
}

/// Converts data in a valid ppm file to an inferred image type
pub fn ppm_to_img(ppm_path: &str, out_path: &str) -> Result<(), Box<dyn Error>> {
    let out_format = infer_type(out_path)?;
    let img = ImageReader::open(ppm_path)?
        .with_guessed_format()?
        .decode()?;
    img.save_with_format(out_path, out_format)?;
    Ok(())
}

/// Converts data in an in-memory buffer of ppm-valid bytes to an inferred image type
pub fn ppm_bytes_to_img(ppm_bytes: &[u8], out_path: &str) -> Result<(), Box<dyn Error>> {
    let out_format = infer_type(out_path)?;
    let img = load_from_memory(ppm_bytes)?;
    img.save_with_format(out_path, out_format)?;
    Ok(())
}

/// Converts an image file (jpg, png, ppm...) to binary PPM (P6) byte buffer
pub fn to_ppm(img_path: &str) -> Result<Vec<u8>, Box<dyn Error>> {
    let path = Path::new(img_path);

    if path.extension().map(|s| s == "ppm").unwrap_or(false) {
        let image = Image::from_file(img_path)?;
        image.bytes_format(PPMFormat::P6)
    } else {
        let img = image::open(img_path)?.to_rgb8();
        let (width, height) = img.dimensions();

        let mut buffer = Vec::new();

        writeln!(buffer, "P6")?;
        writeln!(buffer, "{} {}", width, height)?;
        writeln!(buffer, "255")?;

        for pixel in img.pixels() {
            buffer.write_all(&[pixel[0], pixel[1], pixel[2]])?;
        }

        Ok(buffer)
    }
}

/// Converts any supported image (including PPM) to another format based on output path extension
pub fn convert(input_path: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
    let output_format = infer_type(output_path)?;

    if Path::new(input_path)
        .extension()
        .and_then(|s| s.to_str())
        .map(|ext| ext.eq_ignore_ascii_case("ppm"))
        .unwrap_or(false)
    {
        let image = Image::from_file(input_path)?;
        let bytes = image.bytes_format(PPMFormat::P6)?;
        ppm_bytes_to_img(&bytes, output_path)?;
    } else {
        let img = image::open(input_path)?;
        img.save_with_format(output_path, output_format)?;
    }

    Ok(())
}

impl Image {
    /// Initializes an Image from a valid PPM file
    pub fn from_file(filepath: &str) -> Result<Image, Box<dyn Error>> {
        let mut bytes = to_ppm(filepath)?;
        Self::from_bytes(&mut bytes)
    }

    /// Initializes an Image from the bytes of a PPM file
    pub fn from_bytes(data: &[u8]) -> Result<Image, Box<dyn Error>> {
        let mut cursor = Cursor::new(data);
        Self::from_reader(&mut cursor)
    }

    fn from_reader<R: Read>(reader: &mut R) -> Result<Image, Box<dyn Error>> {
        let mut header = [0; 2];
        reader.read_exact(&mut header)?;

        match &header {
            b"P3" => {
                let buf = io::BufReader::new(reader);
                Self::parse_ppm_ascii(buf.lines())
            }
            b"P6" => Self::parse_ppm_binary(reader),
            _ => Err("Unsupported PPM format".into()),
        }
    }

    fn parse_ppm_ascii<I>(mut lines: I) -> Result<Image, Box<dyn Error>>
    where
        I: Iterator<Item = Result<String, io::Error>>,
    {
        let _magic = lines.next().ok_or("Missing PPM header")??;

        let mut dimensions_line = String::new();
        for line in &mut lines {
            let l = line?;
            if !l.starts_with('#') {
                dimensions_line = l;
                break;
            }
        }

        let mut dims = dimensions_line
            .split_whitespace()
            .map(|s| s.parse::<usize>());
        let width = dims.next().ok_or("Missing width dimension")??;
        let height = dims.next().ok_or("Missing height dimension")??;

        let mut intensity_line = String::new();
        for line in &mut lines {
            let l = line?;
            if !l.starts_with('#') {
                intensity_line = l;
                break;
            }
        }

        let intensity = intensity_line.trim().parse::<usize>()?;

        let pixel_values = lines
            .flat_map(|line| {
                line.ok()
                    .map(|l| l.split_whitespace().map(str::to_string).collect::<Vec<_>>())
            })
            .flatten()
            .map(|s| s.parse::<usize>())
            .collect::<Result<Vec<_>, _>>()?;

        if pixel_values.len() != width * height * 3 {
            return Err("Incorrect number of pixel values".into());
        }

        let mut red_pixels = Vec::with_capacity(width * height);
        let mut blue_pixels = Vec::with_capacity(width * height);
        let mut green_pixels = Vec::with_capacity(width * height);
        for chunk in pixel_values.chunks(3) {
            red_pixels.push(chunk[0]);
            green_pixels.push(chunk[1]);
            blue_pixels.push(chunk[2]);
        }

        Ok(Image {
            width: width,
            height: height,
            max_intensity: intensity,
            red_channel: Matrix::from_vec(width, height, red_pixels)
                .expect("Invalid red channel values"),
            green_channel: Matrix::from_vec(width, height, green_pixels)
                .expect("Invalid green channel values"),
            blue_channel: Matrix::from_vec(width, height, blue_pixels)
                .expect("Invalid blue channel values"),
            format: PPMFormat::P3,
        })
    }

    fn parse_ppm_binary<R: io::Read>(reader: &mut R) -> Result<Image, Box<dyn Error>> {
        let mut buf_reader = io::BufReader::new(reader);

        let mut header = String::new();
        buf_reader.read_line(&mut header)?;

        let dimensions = loop {
            let mut line = String::new();
            buf_reader.read_line(&mut line)?;
            if !line.trim().starts_with('#') {
                let mut parts = line.split_whitespace();
                let w = parts.next().ok_or("Missing width")?.parse::<usize>()?;
                let h = parts.next().ok_or("Missing height")?.parse::<usize>()?;
                break (w, h);
            }
        };

        let (width, height) = dimensions;

        let intensity = loop {
            let mut line = String::new();
            buf_reader.read_line(&mut line)?;
            if !line.trim().starts_with('#') {
                break line.trim().parse::<usize>()?;
            }
        };

        let mut raw = Vec::new();
        buf_reader.read_to_end(&mut raw)?;

        if raw.len() != width * height * 3 {
            return Err("Binary pixel data length mismatch".into());
        }

        let mut red = Vec::with_capacity(width * height);
        let mut green = Vec::with_capacity(width * height);
        let mut blue = Vec::with_capacity(width * height);

        for chunk in raw.chunks_exact(3) {
            red.push(chunk[0] as usize);
            green.push(chunk[1] as usize);
            blue.push(chunk[2] as usize);
        }

        Ok(Image {
            width,
            height,
            max_intensity: intensity,
            red_channel: Matrix::from_vec(width, height, red)
                .ok_or("Invalid red channel values")?,
            green_channel: Matrix::from_vec(width, height, green)
                .ok_or("Invalid green channel values")?,
            blue_channel: Matrix::from_vec(width, height, blue)
                .ok_or("Invalid blue channel values")?,
            format: PPMFormat::P6,
        })
    }

    /// Creates a file and writes the images data to it in valid PPM format
    pub fn write_ppm_file(&self, filepath: &str) -> Result<(), Box<dyn Error>> {
        let file = File::create(filepath)?;
        let mut writer = BufWriter::new(file);

        match self.format {
            PPMFormat::P3 => self.write_ascii(&mut writer),
            PPMFormat::P6 => self.write_binary(&mut writer),
        }
    }

    /// Writes the Image's data to the given writer according to the given ppm format
    pub fn write_ppm_file_format(
        &self,
        filepath: &str,
        format: PPMFormat,
    ) -> Result<(), Box<dyn Error>> {
        let file = File::create(filepath)?;
        let mut writer = BufWriter::new(file);

        match format {
            PPMFormat::P3 => self.write_ascii(&mut writer),
            PPMFormat::P6 => self.write_binary(&mut writer),
        }
    }

    /// Writes the Image's data to the given writer in its internal ppm format
    pub fn write<W: Write>(&self, writer: &mut W) -> Result<(), Box<dyn Error>> {
        match self.format {
            PPMFormat::P3 => self.write_ascii(writer),
            PPMFormat::P6 => self.write_binary(writer),
        }
    }

    /// Writes the Image's data to the given writer according to the given ppm format
    pub fn write_format<W: Write>(
        &self,
        writer: &mut W,
        format: PPMFormat,
    ) -> Result<(), Box<dyn Error>> {
        match format {
            PPMFormat::P3 => self.write_ascii(writer),
            PPMFormat::P6 => self.write_binary(writer),
        }
    }

    /// Gets the Image's data abiding by the internal format
    pub fn bytes(&self) -> Result<Vec<u8>, Box<dyn Error>> {
        let mut buffer = Vec::new();
        match self.format {
            PPMFormat::P3 => self.write_ascii(&mut buffer)?,
            PPMFormat::P6 => self.write_binary(&mut buffer)?,
        }
        Ok(buffer)
    }

    /// Gets the Image's data abiding by the externally provided format
    pub fn bytes_format(&self, format: PPMFormat) -> Result<Vec<u8>, Box<dyn Error>> {
        let mut buffer = Vec::new();

        match format {
            PPMFormat::P3 => self.write_ascii(&mut buffer)?,
            PPMFormat::P6 => self.write_binary(&mut buffer)?,
        }

        Ok(buffer)
    }

    fn write_ascii<W: Write>(&self, writer: &mut W) -> Result<(), Box<dyn Error>> {
        writeln!(writer, "P3")?;
        writeln!(writer, "{} {}", self.width, self.height)?;
        writeln!(writer, "{}", self.max_intensity)?;

        for row in 0..self.height {
            for col in 0..self.width {
                let pixel = self.get_pixel(row, col).unwrap();
                write!(writer, "{} {} {}", pixel.r, pixel.g, pixel.b)?;
                if col < self.width - 1 {
                    write!(writer, " ")?;
                }
            }
            writeln!(writer)?;
        }
        Ok(())
    }

    fn write_binary<W: Write>(&self, writer: &mut W) -> Result<(), Box<dyn Error>> {
        writeln!(writer, "P6")?;
        writeln!(writer, "{} {}", self.width, self.height)?;
        writeln!(writer, "{}", self.max_intensity)?;

        for row in 0..self.height {
            for col in 0..self.width {
                let pixel = self.get_pixel(row, col).unwrap();
                writer.write_all(&[pixel.r as u8, pixel.g as u8, pixel.b as u8])?;
            }
        }
        Ok(())
    }
}

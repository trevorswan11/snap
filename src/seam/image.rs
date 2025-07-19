use std::error::Error;
use std::fs::File;
use std::io::{self, BufRead, BufWriter, Cursor, Read, Write};

use crate::seam::matrix::*;

/// Representation of an RGB Pixel
#[derive(Debug)]
pub struct PixelRGB {
    pub r: usize,
    pub g: usize,
    pub b: usize,
}

/// Represents the two common types of PPM files
#[derive(Debug)]
pub enum PPMFormat {
    P3,
    P6,
}

/// Representation of a 2D RGB image
#[derive(Debug)]
pub struct Image {
    pub width: usize,
    pub height: usize,
    pub max_intensity: usize,
    red_channel: Matrix<usize>,
    blue_channel: Matrix<usize>,
    green_channel: Matrix<usize>,
    format: PPMFormat,
}

impl Image {
    /// Initializes an Image with the given width, height, and intensity with all channels set to 0
    pub fn new(width: usize, height: usize, intensity: usize, format: PPMFormat) -> Image {
        Image {
            width: width,
            height: height,
            max_intensity: intensity,
            red_channel: Matrix::new_filled(width, height, 0),
            blue_channel: Matrix::new_filled(width, height, 0),
            green_channel: Matrix::new_filled(width, height, 0),
            format: format,
        }
    }

    /// Initializes an Image from a valid PPM file
    pub fn from_file(filepath: &str) -> Result<Image, Box<dyn Error>> {
        let mut file = File::open(filepath)?;
        Self::from_reader(&mut file)
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

    /// Writes the Image's data to the given writer in valid ppm format
    pub fn write_to<W: Write>(&self, writer: &mut W) -> Result<(), Box<dyn Error>> {
        match self.format {
            PPMFormat::P3 => self.write_ascii(writer),
            PPMFormat::P6 => self.write_binary(writer),
        }
    }

    pub fn to_bytes(&self) -> Result<Vec<u8>, Box<dyn Error>> {
        let mut buffer = Vec::new();
        match self.format {
            PPMFormat::P3 => self.write_ascii(&mut buffer)?,
            PPMFormat::P6 => self.write_binary(&mut buffer)?,
        };
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

    /// Returns the { r, g, b } pixel at the given row and col in the Image
    pub fn get_pixel(&self, row: usize, col: usize) -> Option<PixelRGB> {
        if row < self.height && col < self.width {
            Some(PixelRGB {
                r: self.red_channel[(row, col)],
                g: self.green_channel[(row, col)],
                b: self.blue_channel[(row, col)],
            })
        } else {
            None
        }
    }

    /// Sets the pixel in the Image at the given row and column to the given color
    pub fn set_pixel(&mut self, row: usize, col: usize, color: PixelRGB) {
        if row < self.height && col < self.width {
            self.red_channel[(row, col)] = color.r;
            self.green_channel[(row, col)] = color.g;
            self.blue_channel[(row, col)] = color.b;
        }
    }

    /// Sets each pixel in the image to the given color
    pub fn fill(&mut self, color: PixelRGB) {
        self.red_channel.fill(color.r);
        self.green_channel.fill(color.g);
        self.blue_channel.fill(color.b);
    }

    /// The image is rotated 90 degrees to the left (counterclockwise)
    pub fn rotate_left(&mut self) {
        if self.width == 0 || self.height == 0 {
            return;
        }

        let (width, height) = (self.width, self.height);

        let mut new_red = Matrix::new_filled(height, width, 0);
        let mut new_green = Matrix::new_filled(height, width, 0);
        let mut new_blue = Matrix::new_filled(height, width, 0);

        for row in 0..height {
            for col in 0..width {
                let pixel = self.get_pixel(row, col).expect("Invalid pixel coordinate");
                let new_row = width - 1 - col;
                let new_col = row;

                new_red[(new_row, new_col)] = pixel.r;
                new_green[(new_row, new_col)] = pixel.g;
                new_blue[(new_row, new_col)] = pixel.b;
            }
        }

        self.red_channel = new_red;
        self.green_channel = new_green;
        self.blue_channel = new_blue;
        self.height = width;
        self.width = height;
    }

    /// The image is rotated 90 degrees to the right (clockwise)
    pub fn rotate_right(&mut self) {
        if self.width == 0 || self.height == 0 {
            return;
        }

        let (width, height) = (self.width, self.height);

        let mut new_red = Matrix::new_filled(height, width, 0);
        let mut new_green = Matrix::new_filled(height, width, 0);
        let mut new_blue = Matrix::new_filled(height, width, 0);

        for row in 0..height {
            for col in 0..width {
                let pixel = self.get_pixel(row, col).expect("Invalid pixel coordinate");
                let new_row = col;
                let new_col = height - 1 - row;

                new_red[(new_row, new_col)] = pixel.r;
                new_green[(new_row, new_col)] = pixel.g;
                new_blue[(new_row, new_col)] = pixel.b;
            }
        }

        self.red_channel = new_red;
        self.green_channel = new_green;
        self.blue_channel = new_blue;
        self.height = width;
        self.width = height;
    }

    /// Computes the energy matrix for the image
    pub fn energy(&self) -> Matrix<isize> {
        let mut energy = Matrix::new_filled(self.width, self.height, 0);
        let mut max_energy = 0;

        for row in 1..self.height - 1 {
            for col in 1..self.width - 1 {
                let n = self.get_pixel(row - 1, col).unwrap();
                let s = self.get_pixel(row + 1, col).unwrap();
                let e = self.get_pixel(row, col + 1).unwrap();
                let w = self.get_pixel(row, col - 1).unwrap();

                let energy_val = n.squared_difference(&s) + e.squared_difference(&w);
                energy[(row, col)] = energy_val;

                max_energy = if energy_val > max_energy {
                    energy_val
                } else {
                    max_energy
                }
            }
        }

        if max_energy == 0 {
            max_energy = 1;
        }
        energy.fill_border(max_energy);

        energy
    }

    /// Computes the vertical cost matrix for the image
    pub fn vertical_cost(&self) -> Matrix<isize> {
        let energy = self.energy();
        let mut cost = Matrix::new_filled(self.width, self.height, 0);

        for col in 0..self.width {
            cost[(0, col)] = energy[(0, col)];
        }

        for row in 1..self.height {
            for col in 0..self.width {
                let mut min_prev = cost[(row - 1, col)];

                if col > 0 {
                    min_prev = min_prev.min(cost[(row - 1, col - 1)]);
                }
                if col < self.width - 1 {
                    min_prev = min_prev.min(cost[(row - 1, col + 1)]);
                }

                cost[(row, col)] = energy[(row, col)] + min_prev;
            }
        }
        cost
    }

    /// Returns the vertical seam with the minimal cost
    pub fn minimal_vertical_seam(&self) -> Vec<usize> {
        let cost = self.vertical_cost();
        let mut seam = vec![0; self.height];

        let mut current_col = cost
            .min_in_row_range(self.height - 1, 0, self.width)
            .expect("Bottom row should not be empty")
            .0;

        seam[self.height - 1] = current_col;

        for row in (0..self.height - 1).rev() {
            let start = current_col.saturating_sub(1);
            let end = (current_col + 2).min(self.width);

            current_col = cost
                .min_in_row_range(row, start, end)
                .expect("No valid columns in range")
                .0;

            seam[row] = current_col;
        }

        seam
    }

    /// Removes the minimal vertical seam from the image
    pub fn remove_vertical_seam(&mut self) {
        let seam = self.minimal_vertical_seam();
        assert_eq!(seam.len(), self.height, "Seam must have one entry per row");

        for row in 0..self.height {
            let seam_col = seam[row];

            assert!(
                seam_col < self.width,
                "Invalid seam column {} at row {}, exceeds image width {}",
                seam_col,
                row,
                self.width
            );

            for col in seam_col..self.width - 1 {
                self.red_channel[(row, col)] = self.red_channel[(row, col + 1)];
                self.green_channel[(row, col)] = self.green_channel[(row, col + 1)];
                self.blue_channel[(row, col)] = self.blue_channel[(row, col + 1)];
            }
        }

        self.width -= 1;
        self.red_channel.trim_width(self.width);
        self.green_channel.trim_width(self.width);
        self.blue_channel.trim_width(self.width);
    }

    /// Reduces the width of the Image to be the new width
    pub fn seam_carve_width(&mut self, new_width: usize) {
        if self.width == new_width {
            return;
        }

        for _ in 0..(self.width.saturating_sub(new_width)) {
            self.remove_vertical_seam();
        }
    }

    /// Reduces the height of the Image to be the new height
    pub fn seam_carve_height(&mut self, new_height: usize) {
        self.rotate_left();
        self.seam_carve_width(new_height);
        self.rotate_right();
    }

    /// Reduces the width and height of the Image to the given values
    pub fn seam_carve(&mut self, new_width: usize, new_height: usize) {
        self.seam_carve_width(new_width);
        self.seam_carve_height(new_height);
    }
}

impl PixelRGB {
    pub fn squared_difference(&self, other: &PixelRGB) -> isize {
        let dr: isize = self.r as isize - other.r as isize;
        let dg: isize = self.g as isize - other.g as isize;
        let db: isize = self.b as isize - other.b as isize;
        dr * dr + dg * dg + db * db
    }
}

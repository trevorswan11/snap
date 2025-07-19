use std::error::Error;
use std::fs::File;
use std::io::{self, BufRead, BufWriter, Write};

use crate::seam::matrix::*;

/// Representation of an RGB Pixel
#[derive(Debug)]
pub struct PixelRGB {
    pub r: usize,
    pub g: usize,
    pub b: usize,
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
}

impl Image {
    /// Initializes an Image with the given width, height, and intensity with all channels set to 0
    pub fn new(width: usize, height: usize, intensity: usize) -> Image {
        Image {
            width: width,
            height: height,
            max_intensity: intensity,
            red_channel: Matrix::new_filled(width, height, 0),
            blue_channel: Matrix::new_filled(width, height, 0),
            green_channel: Matrix::new_filled(width, height, 0),
        }
    }

    /// Initializes an Image from a valid PPM
    pub fn from_ppm(filepath: &str) -> Result<Image, Box<dyn Error>> {
        let f = File::open(filepath)?;
        let reader = io::BufReader::new(f);
        let mut lines = reader.lines();

        // Verify the 'P3' header
        let magic = lines.next().ok_or("Missing PPM header")??;
        if magic.trim().to_uppercase() != "P3" {
            return Err("Invalid PPM format header".into());
        }

        // Get dims, but skip any present comments
        let mut dimensions_line = String::new();
        for line in &mut lines {
            let l = line?;
            if !l.starts_with('#') {
                dimensions_line = l;
                break;
            }
        }

        // Unpack the dimensions
        let mut dims = dimensions_line
            .split_whitespace()
            .map(|s| s.parse::<usize>());
        let width = dims.next().ok_or("Missing width dimension")??;
        let height = dims.next().ok_or("Missing height dimension")??;

        // Get the max intensity value, skipping comments
        let mut intensity_line = String::new();
        for line in &mut lines {
            let l = line?;
            if !l.starts_with('#') {
                intensity_line = l;
                break;
            }
        }

        // Unpack the intensity
        let intensity = intensity_line.trim().parse::<usize>()?;

        // Read the pixels
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

        // Chunk the pixels and pack into rgb vec
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
        })
    }

    pub fn write_ppm_file(&self, filepath: &str) -> Result<(), Box<dyn Error>> {
        let file = File::create(filepath)?;
        let mut writer = BufWriter::new(file);

        // Write PPM header
        writeln!(writer, "P3")?;
        writeln!(writer, "{} {}", self.width, self.height)?;
        writeln!(writer, "{}", self.max_intensity)?;

        // Write pixel data: one row per line
        for row in 0..self.height {
            for col in 0..self.width {
                let pixel = self.get_pixel(row, col).unwrap();
                write!(writer, "{} {} {}", pixel.r, pixel.g, pixel.b)?;
                if col < self.width - 1 {
                    write!(writer, " ")?; // space between pixels
                }
            }
            writeln!(writer)?; // newline after each row
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

        // The first row is just the energy values
        for col in 0..self.width {
            cost[(0, col)] = energy[(0, col)];
        }

        // Iteratively fill in the rest of the cost matrix
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

    /// Returns the vertical seam with the minimal cost according to the given
    /// cost matrix, represented as a vector filled with the column numbers for
    /// each pixel along the seam, starting with the lowest numbered row (top
    /// of image) and progressing to the highest (bottom of image). The length
    /// of the returned vector is equal to the height of the cost matrix.
    /// While determining the seam, if any pixels tie for lowest cost, the
    /// leftmost one (i.e. with the lowest column number) is used.
    pub fn minimal_vertical_seam(&self) -> Vec<usize> {
        let cost = self.vertical_cost();
        let mut seam = vec![0; self.height];

        // Start at the minimum in the bottom row
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

    /// Removes the given vertical seam from the Image. That is, one
    /// pixel will be removed from every row in the image. The pixel
    /// removed from row r will be the one with column equal to seam[r].
    /// The width of the image will be one less than before.
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

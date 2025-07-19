use std::{fs::File};
use std::io::{self, BufRead};
use std::error:: Error;

use crate::seam::matrix::*;

/// Representation of an RGB Pixel
pub struct Pixel3 {
    r: usize,
    g: usize,
    b: usize,
}

#[derive(Debug)]
/// Representation of a 2D RGB image
pub struct Image {
    width: usize,
    height: usize,
    max_intensity: usize,
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
            red_channel: Matrix::<usize>::new_filled(width, height, 0),
            blue_channel: Matrix::<usize>::new_filled(width, height, 0),
            green_channel: Matrix::<usize>::new_filled(width, height, 0),
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
        let mut dimensions = String::new();
        for line in 
    }
}

use crate::img::crop::CropMethod;
use crate::img::io::{PPMFormat, ppm_bytes_to_img};
use crate::img::matrix::*;
use crate::img::scale::ScaleMethod;
use crate::img::utils::PixelRGB;

use std::error::Error;
use std::fs;
use std::path::Path;

/// Representation of a 2D RGB image
#[derive(Debug)]
pub struct Image {
    pub width: usize,
    pub height: usize,
    pub max_intensity: usize,
    pub red_channel: Matrix<usize>,
    pub blue_channel: Matrix<usize>,
    pub green_channel: Matrix<usize>,
    pub format: PPMFormat,
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

    pub fn resize(
        &mut self,
        target_width: usize,
        target_height: usize,
        method: ScaleMethod,
        crop_x: Option<CropMethod>,
        crop_y: Option<CropMethod>,
    ) {
        if target_width > self.width {
            self.scale(target_width, self.height, method.clone());
        } else if target_width < self.width {
            let crop_method = crop_x.expect("Crop method for the x-axis needed for this resize");
            self.crop_width(target_width, crop_method);
        }

        if target_height > self.height {
            self.scale(self.width, target_height, method);
        } else if target_height < self.height {
            let crop_method = crop_y.expect("Crop method for the y-axis needed for this resize");
            self.crop_height(target_height, crop_method);
        }
    }

    /// Scales the image up to a higher width and height
    pub fn scale(&mut self, new_width: usize, new_height: usize, method: ScaleMethod) {
        if self.width == 0 || self.height == 0 || new_width == 0 || new_height == 0 {
            return;
        }

        match method {
            ScaleMethod::Linear => self.linear_scale(new_width, new_height),
            ScaleMethod::Bilinear => self.bilinear_scale(new_width, new_height),
        }
    }

    /// Crops the image using the given cropping method
    pub fn crop(&mut self, new_width: usize, new_height: usize, method: CropMethod, rect_center_x: Option<usize>, rect_center_y: Option<usize>) {
        if new_width == 0 || new_height == 0 || new_width > self.width || new_height > self.height {
            return;
        }

        let w_diff = self.width - new_width;
        let h_diff = self.height - new_height;

        match method {
            CropMethod::Left => {
                self.crop_left(new_width);
            }
            CropMethod::Right => {
                self.crop_right(new_width);
            }
            CropMethod::LeftRight => {
                let left_trim = w_diff / 2;
                let right_trim = w_diff - left_trim;
                self.crop_right(self.width - right_trim);
                self.crop_left(new_width);
            }

            CropMethod::Top => {
                self.crop_top(new_height);
            }
            CropMethod::Bottom => {
                self.crop_bottom(new_height);
            }
            CropMethod::TopBottom => {
                let top_trim = h_diff / 2;
                let bottom_trim = h_diff - top_trim;
                self.crop_bottom(self.height - bottom_trim);
                self.crop_top(new_height);
            }

            CropMethod::LeftTop => {
                self.crop_left(new_width);
                self.crop_top(new_height);
            }
            CropMethod::LeftBottom => {
                self.crop_left(new_width);
                self.crop_bottom(new_height);
            }
            CropMethod::RightTop => {
                self.crop_right(new_width);
                self.crop_top(new_height);
            }
            CropMethod::RightBottom => {
                self.crop_right(new_width);
                self.crop_bottom(new_height);
            }

            CropMethod::Rectangular => {
                let x_offset = rect_center_x.unwrap_or((self.width - new_width) / 2);
                let y_offset = rect_center_y.unwrap_or((self.height - new_height) / 2);

                self.crop_rect(new_width, new_height, x_offset, y_offset);
            }
        }
    }

    /// Reduces the width and height of the Image to the given values
    pub fn seam_carve(&mut self, new_width: usize, new_height: usize) {
        self.seam_carve_width(new_width);
        self.seam_carve_height(new_height);
    }

    /// Multiplies each pixels { r, g, b } values by the given scalars. Clamps to [0, 1]
    pub fn scale_rgb(
        &mut self,
        r_scale: f64,
        g_scale: f64,
        b_scale: f64,
    ) -> Result<(), Box<dyn Error>> {
        let (r_scale, g_scale, b_scale) = (
            r_scale.clamp(0.0, 1.0),
            g_scale.clamp(0.0, 1.0),
            b_scale.clamp(0.0, 1.0),
        );

        for row in 0..self.height {
            for col in 0..self.width {
                let curr_color = self
                    .get_pixel(row, col)
                    .ok_or("Pixel indices out of bounds")?;
                let new_color = PixelRGB {
                    r: (curr_color.r as f64 * r_scale) as usize,
                    g: (curr_color.g as f64 * g_scale) as usize,
                    b: (curr_color.b as f64 * b_scale) as usize,
                };
                self.set_pixel(row, col, new_color);
            }
        }
        Ok(())
    }

    /// Shifts the hue of every pixel by the given degrees (0–360), wraps around the color wheel.
    pub fn hue_shift(&mut self, degrees: f64) -> Result<(), Box<dyn std::error::Error>> {
        for row in 0..self.height {
            for col in 0..self.width {
                let pixel = self.get_pixel(row, col).ok_or("Pixel out of bounds")?;
                let (h, s, l) = Self::rgb_to_hsl(pixel.r as f64, pixel.g as f64, pixel.b as f64);

                let new_h = (h + degrees) % 360.0;
                let (r, g, b) = Self::hsl_to_rgb(new_h, s, l);

                self.set_pixel(row, col, PixelRGB { r, g, b });
            }
        }
        Ok(())
    }

    /// Mirrors the images pixel maps about the horizontal axis
    pub fn mirror_x(&mut self) {
        self.red_channel.mirror_x();
        self.green_channel.mirror_x();
        self.blue_channel.mirror_x();
    }

    /// Mirrors the images pixel maps about the vertical axis
    pub fn mirror_y(&mut self) {
        self.red_channel.mirror_y();
        self.green_channel.mirror_y();
        self.blue_channel.mirror_y();
    }

    /// Transposes the image
    pub fn transpose(&mut self) {
        self.red_channel.transpose();
        self.green_channel.transpose();
        self.blue_channel.transpose();
        std::mem::swap(&mut self.width, &mut self.height);
    }

    /// Saves the image to a file with the filetype inferred from the output path
    pub fn save(&self, output_path: &str) -> Result<(), Box<dyn Error>> {
        let path = Path::new(output_path);

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let bytes = self.bytes_format(PPMFormat::P6)?;
        ppm_bytes_to_img(&bytes, output_path)?;
        Ok(())
    }
}

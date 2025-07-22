use crate::img::image::*;
use crate::img::matrix::*;

use clap::ValueEnum;

/// Method to use when cropping image
#[derive(Debug, Clone, ValueEnum)]
#[clap(rename_all = "kebab_case")]
pub enum CropMethod {
    Left,
    Right,
    LeftRight,

    Top,
    Bottom,
    TopBottom,

    LeftTop,
    LeftBottom,

    RightTop,
    RightBottom,

    Rectangular
}

impl Image {
    pub fn crop_left(&mut self, new_width: usize) {
        if new_width >= self.width || new_width == 0 {
            return;
        }

        let cols_to_trim = self.width - new_width;

        for row in 0..self.height {
            for col in 0..new_width {
                self.red_channel[(row, col)] = self.red_channel[(row, col + cols_to_trim)];
                self.green_channel[(row, col)] = self.green_channel[(row, col + cols_to_trim)];
                self.blue_channel[(row, col)] = self.blue_channel[(row, col + cols_to_trim)];
            }
        }

        self.width = new_width;
        self.red_channel.trim_width(new_width);
        self.green_channel.trim_width(new_width);
        self.blue_channel.trim_width(new_width);
    }

    pub fn crop_right(&mut self, new_width: usize) {
        if new_width >= self.width || new_width == 0 {
            return;
        }

        self.width = new_width;
        self.red_channel.trim_width(new_width);
        self.green_channel.trim_width(new_width);
        self.blue_channel.trim_width(new_width);
    }

    pub fn crop_top(&mut self, new_height: usize) {
        if new_height >= self.height || new_height == 0 {
            return;
        }

        let rows_to_trim = self.height - new_height;

        for row in 0..new_height {
            for col in 0..self.width {
                self.red_channel[(row, col)] = self.red_channel[(row + rows_to_trim, col)];
                self.green_channel[(row, col)] = self.green_channel[(row + rows_to_trim, col)];
                self.blue_channel[(row, col)] = self.blue_channel[(row + rows_to_trim, col)];
            }
        }

        self.height = new_height;
        self.red_channel.datum.truncate(new_height * self.width);
        self.green_channel.datum.truncate(new_height * self.width);
        self.blue_channel.datum.truncate(new_height * self.width);
    }

    pub fn crop_bottom(&mut self, new_height: usize) {
        if new_height >= self.height || new_height == 0 {
            return;
        }

        self.height = new_height;
        self.red_channel.datum.truncate(new_height * self.width);
        self.green_channel.datum.truncate(new_height * self.width);
        self.blue_channel.datum.truncate(new_height * self.width);
    }

    pub fn crop_rect(
        &mut self,
        new_width: usize,
        new_height: usize,
        x_offset: usize,
        y_offset: usize,
    ) {
        let mut new_red = Matrix::new_filled(new_width, new_height, 0);
        let mut new_green = Matrix::new_filled(new_width, new_height, 0);
        let mut new_blue = Matrix::new_filled(new_width, new_height, 0);

        for row in 0..new_height {
            for col in 0..new_width {
                new_red[(row, col)] = self.red_channel[(y_offset + row, x_offset + col)];
                new_green[(row, col)] = self.green_channel[(y_offset + row, x_offset + col)];
                new_blue[(row, col)] = self.blue_channel[(y_offset + row, x_offset + col)];
            }
        }

        self.width = new_width;
        self.height = new_height;
        self.red_channel = new_red;
        self.green_channel = new_green;
        self.blue_channel = new_blue;
    }

    pub fn crop_width(&mut self, new_width: usize, method: CropMethod) {
        match method {
            CropMethod::Left => self.crop_left(new_width),
            CropMethod::Right => self.crop_right(new_width),
            CropMethod::LeftRight => {
                // Crop evenly from both sides
                let total_trim = self.width.saturating_sub(new_width);
                let left_trim = total_trim / 2;
                let new_x_offset = left_trim;

                self.crop_rect(new_width, self.height, new_x_offset, 0);
            }
            _ => panic!("Invalid crop method for width"),
        }
    }

    pub fn crop_height(&mut self, new_height: usize, method: CropMethod) {
        match method {
            CropMethod::Top => self.crop_top(new_height),
            CropMethod::Bottom => self.crop_bottom(new_height),
            CropMethod::TopBottom => {
                let total_trim = self.height.saturating_sub(new_height);
                let top_trim = total_trim / 2;
                let new_y_offset = top_trim;

                self.crop_rect(self.width, new_height, 0, new_y_offset);
            }
            _ => panic!("Invalid crop method for height"),
        }
    }
}

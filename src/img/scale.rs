use crate::img::image::*;
use crate::img::matrix::*;

use clap::ValueEnum;

/// Options available for Scaling UP an image
#[derive(Debug, Clone, ValueEnum)]
#[clap(rename_all = "kebab_case")]
pub enum ScaleMethod {
    Linear,
    Bilinear,
}

impl Image {
    pub fn linear_scale(&mut self, new_width: usize, new_height: usize) {
        let mut new_red = Matrix::new_filled(new_width, new_height, 0);
        let mut new_green = Matrix::new_filled(new_width, new_height, 0);
        let mut new_blue = Matrix::new_filled(new_width, new_height, 0);

        for new_row in 0..new_height {
            for new_col in 0..new_width {
                let orig_row = new_row * self.height / new_height;
                let orig_col = new_col * self.width / new_width;

                let pixel = self.get_pixel(orig_row, orig_col).unwrap();
                new_red[(new_row, new_col)] = pixel.r;
                new_green[(new_row, new_col)] = pixel.g;
                new_blue[(new_row, new_col)] = pixel.b;
            }
        }

        self.width = new_width;
        self.height = new_height;
        self.red_channel = new_red;
        self.green_channel = new_green;
        self.blue_channel = new_blue;
    }

    pub fn bilinear_scale(&mut self, new_width: usize, new_height: usize) {
        let mut new_red = Matrix::new_filled(new_width, new_height, 0);
        let mut new_green = Matrix::new_filled(new_width, new_height, 0);
        let mut new_blue = Matrix::new_filled(new_width, new_height, 0);

        for new_y in 0..new_height {
            for new_x in 0..new_width {
                // Map the target pixel (new_x, new_y) to source image space
                let src_x = (new_x as f64) * (self.width as f64) / (new_width as f64);
                let src_y = (new_y as f64) * (self.height as f64) / (new_height as f64);

                let x0 = src_x.floor() as usize;
                let x1 = x0.min(self.width - 1).saturating_add(1).min(self.width - 1);
                let y0 = src_y.floor() as usize;
                let y1 = y0
                    .min(self.height - 1)
                    .saturating_add(1)
                    .min(self.height - 1);

                let dx = src_x - x0 as f64;
                let dy = src_y - y0 as f64;

                let p00 = self.get_pixel(y0, x0).unwrap();
                let p10 = self.get_pixel(y0, x1).unwrap();
                let p01 = self.get_pixel(y1, x0).unwrap();
                let p11 = self.get_pixel(y1, x1).unwrap();

                let interpolate =
                    |a, b, t: f64| (a as f64 * (1.0 - t) + b as f64 * t).round() as usize;

                let r_top = interpolate(p00.r, p10.r, dx);
                let r_bottom = interpolate(p01.r, p11.r, dx);
                let r = interpolate(r_top, r_bottom, dy);

                let g_top = interpolate(p00.g, p10.g, dx);
                let g_bottom = interpolate(p01.g, p11.g, dx);
                let g = interpolate(g_top, g_bottom, dy);

                let b_top = interpolate(p00.b, p10.b, dx);
                let b_bottom = interpolate(p01.b, p11.b, dx);
                let b = interpolate(b_top, b_bottom, dy);

                new_red[(new_y, new_x)] = r;
                new_green[(new_y, new_x)] = g;
                new_blue[(new_y, new_x)] = b;
            }
        }

        self.width = new_width;
        self.height = new_height;
        self.red_channel = new_red;
        self.green_channel = new_green;
        self.blue_channel = new_blue;
    }
}

use crate::img::image::*;

/// Representation of an RGB Pixel
#[derive(Debug)]
pub struct PixelRGB {
    pub r: usize,
    pub g: usize,
    pub b: usize,
}

impl Image {
    pub fn fill(&mut self, color: PixelRGB) {
        self.red_channel.fill(color.r);
        self.green_channel.fill(color.g);
        self.blue_channel.fill(color.b);
    }

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

    pub fn set_pixel(&mut self, row: usize, col: usize, color: PixelRGB) {
        if row < self.height && col < self.width {
            self.red_channel[(row, col)] = color.r;
            self.green_channel[(row, col)] = color.g;
            self.blue_channel[(row, col)] = color.b;
        }
    }

    pub fn rgb_to_hsl(r: f64, g: f64, b: f64) -> (f64, f64, f64) {
        let r = r / 255.0;
        let g = g / 255.0;
        let b = b / 255.0;

        let max = r.max(g).max(b);
        let min = r.min(g).min(b);
        let delta = max - min;

        let l = (max + min) / 2.0;

        let s = if delta == 0.0 {
            0.0
        } else {
            delta / (1.0 - (2.0 * l - 1.0).abs())
        };

        let h = if delta == 0.0 {
            0.0
        } else if max == r {
            60.0 * (((g - b) / delta) % 6.0)
        } else if max == g {
            60.0 * (((b - r) / delta) + 2.0)
        } else {
            60.0 * (((r - g) / delta) + 4.0)
        };

        let h = if h < 0.0 { h + 360.0 } else { h };
        (h, s, l)
    }

    pub fn hsl_to_rgb(h: f64, s: f64, l: f64) -> (usize, usize, usize) {
        let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
        let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
        let m = l - c / 2.0;

        let (r1, g1, b1) = match h {
            h if h < 60.0 => (c, x, 0.0),
            h if h < 120.0 => (x, c, 0.0),
            h if h < 180.0 => (0.0, c, x),
            h if h < 240.0 => (0.0, x, c),
            h if h < 300.0 => (x, 0.0, c),
            _ => (c, 0.0, x),
        };

        let to_255 = |v: f64| ((v + m) * 255.0).round().clamp(0.0, 255.0) as usize;

        (to_255(r1), to_255(g1), to_255(b1))
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

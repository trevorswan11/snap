use crate::img::image::*;
use crate::img::matrix::*;

impl Image {
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

    pub fn seam_carve_width(&mut self, new_width: usize) {
        if self.width == new_width {
            return;
        }

        for _ in 0..(self.width.saturating_sub(new_width)) {
            self.remove_vertical_seam();
        }
    }

    pub fn seam_carve_height(&mut self, new_height: usize) {
        self.rotate_left();
        self.seam_carve_width(new_height);
        self.rotate_right();
    }
}

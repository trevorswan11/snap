use std::fmt;
use std::ops::{Index, IndexMut};

/// A generic matrix type
#[derive(Clone)]
pub struct Matrix<T> {
    pub width: usize,
    pub height: usize,
    pub datum: Vec<T>,
}

impl<T> Matrix<T>
where
    T: Copy + Clone + Ord,
{
    pub fn new(width: usize, height: usize) -> Matrix<T>
    where
        T: Default,
    {
        let datum = vec![T::default(); width * height];
        Matrix {
            width,
            height,
            datum,
        }
    }

    pub fn new_filled(width: usize, height: usize, value: T) -> Matrix<T> {
        let datum = vec![value; width * height];
        Matrix {
            width,
            height,
            datum,
        }
    }

    pub fn from_vec(width: usize, height: usize, data: Vec<T>) -> Option<Self> {
        if data.len() != width * height {
            return None;
        } else {
            Some(Self {
                width,
                height,
                datum: data,
            })
        }
    }

    /// Less idiomatic way to get a reference to a stored value. Use Index Trait
    pub fn get(&self, row: usize, col: usize) -> Option<&T> {
        if row < self.height && col < self.width {
            Some(&self[(row, col)])
        } else {
            None
        }
    }

    /// Less idiomatic way to set a reference to a stored value. Use IndexMut Trait
    pub fn set(&mut self, row: usize, col: usize, value: T) {
        if row < self.height && col < self.width {
            self[(row, col)] = value;
        }
    }

    pub fn fill(&mut self, value: T) {
        self.datum.fill(value);
    }

    pub fn fill_border(&mut self, value: T) {
        let (width, height) = (self.width, self.height);

        for col in 0..width {
            self[(0, col)] = value.clone();
            self[(height - 1, col)] = value.clone();
        }

        for row in 0..height {
            self[(row, 0)] = value.clone();
            self[(row, width - 1)] = value.clone();
        }
    }

    pub fn min(&self) -> Option<T> {
        self.datum.iter().copied().min()
    }

    pub fn max(&self) -> Option<T> {
        self.datum.iter().copied().max()
    }

    /// Returns minimum in row as: (index, val)
    pub fn min_in_row(&self, row: usize) -> Option<(usize, T)> {
        self.min_in_row_range(row, 0, self.width)
    }

    /// Returns minimum in row's range as: (index, val)
    pub fn min_in_row_range(
        &self,
        row: usize,
        column_start: usize,
        column_end: usize,
    ) -> Option<(usize, T)> {
        if row >= self.height || column_start >= column_end || column_end > self.width {
            return None;
        }

        let mut min_val = self[(row, column_start)];
        let mut min_index = column_start;

        for col in (column_start + 1)..column_end {
            let val = self[(row, col)];
            if val < min_val {
                min_val = val;
                min_index = col;
            }
        }

        Some((min_index, min_val))
    }

    pub fn trim_width(&mut self, new_width: usize) {
        assert!(new_width <= self.width);

        let mut new_datum = Vec::with_capacity(self.height * new_width);

        for row in 0..self.height {
            let old_row_start = row * self.width;
            let old_row_end = old_row_start + self.width;
            let row_slice = &self.datum[old_row_start..old_row_end];
            new_datum.extend_from_slice(&row_slice[..new_width]);
        }

        self.datum = new_datum;
        self.width = new_width;
    }

    pub fn transpose(&mut self) {
        let mut new_data = vec![self.datum[0]; self.width * self.height];
        for row in 0..self.height {
            for col in 0..self.width {
                new_data[col * self.height + row] = self[(row, col)];
            }
        }
        std::mem::swap(&mut self.width, &mut self.height);
        self.datum = new_data;
    }

    pub fn mirror_y(&mut self) {
        for row in 0..self.height {
            for col in 0..self.width / 2 {
                self.datum.swap(
                    row * self.width + col,
                    row * self.width + (self.width - 1 - col),
                );
            }
        }
    }

    pub fn mirror_x(&mut self) {
        for col in 0..self.width {
            for row in 0..self.height / 2 {
                let top = row * self.width + col;
                let bottom = (self.height - 1 - row) * self.width + col;
                self.datum.swap(top, bottom);
            }
        }
    }
}

impl<T> Index<(usize, usize)> for Matrix<T> {
    type Output = T;

    fn index(&self, (row, col): (usize, usize)) -> &Self::Output {
        &self.datum[row * self.width + col]
    }
}

impl<T> IndexMut<(usize, usize)> for Matrix<T> {
    fn index_mut(&mut self, (row, col): (usize, usize)) -> &mut Self::Output {
        &mut self.datum[row * self.width + col]
    }
}

impl<T: fmt::Debug> fmt::Debug for Matrix<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "Matrix<{}> ({} x {}):",
            std::any::type_name::<T>(),
            self.height,
            self.width
        )?;
        for row in 0..self.height {
            let start = row * self.width;
            let end = start + self.width;
            for col in &self.datum[start..end] {
                write!(f, "{:?} ", col)?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

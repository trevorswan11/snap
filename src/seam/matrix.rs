#![allow(dead_code)]

use std::fmt;
use std::ops::{Index, IndexMut};

/// A generic matrix type
#[derive(Clone)]
pub struct Matrix<T> {
    pub width: usize,
    pub height: usize,
    datum: Vec<T>,
}

impl<T> Matrix<T>
where
    T: Copy + Clone + Ord,
{
    /// Creates an empty matrix with specified width and height
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

    /// Sets each element of the Matrix to the given value
    pub fn fill(&mut self, value: T) {
        self.datum.fill(value);
    }

    /// Sets each element on the border of the Matrix to the given value
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

    /// Returns the minimum value in the matrix
    pub fn min(&self) -> Option<T> {
        self.datum.iter().copied().min()
    }

    /// Returns the maximum value in the matrix
    pub fn max(&self) -> Option<T> {
        self.datum.iter().copied().max()
    }

    /// Returns a pair of the minimal value in and the column where the element
    /// with the minimal value in a particular row is located. Fmt: (index, val)
    pub fn min_in_row(&self, row: usize) -> Option<(usize, T)> {
        self.min_in_row_range(row, 0, self.width)
    }

    /// Returns a pair of the minimal value in and the column where the element
    /// with the minimal value in a particular region in a given row is located.
    /// Fmt: (index, val)
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

    /// Truncates the matrix to the new width. Asserts that the matrix will shrink
    pub fn trim_width(&mut self, new_width: usize) {
        assert!(new_width <= self.width);

        let mut new_datum = Vec::with_capacity(self.height * new_width);

        for row in 0..self.height {
            let start = row * self.width;
            let end = start + new_width;
            new_datum.extend_from_slice(&self.datum[start..end]);
        }

        self.datum = new_datum;
        self.width = new_width;
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

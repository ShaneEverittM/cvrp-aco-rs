use std::ops::{Index, IndexMut};

#[derive(Default, Debug)]
pub struct Matrix {
    dimension: usize,
    data: Vec<f64>,
}

impl Matrix {
    pub fn new(dimension: usize) -> Self {
        Self {
            dimension,
            data: vec![0.0; dimension * dimension],
        }
    }

    pub fn update<F: FnOnce(f64) -> f64>(&mut self, i: usize, j: usize, op: F) {
        self[i][j] = op(self[i][j])
    }

    pub fn filled_with(dimension: usize, value: f64) -> Self {
        let mut matrix = Self::new(dimension);
        for i in 0..dimension {
            for j in 0..dimension {
                matrix[i][j] = value
            }
        }
        matrix
    }

    pub fn size(&self) -> usize {
        self.dimension
    }
}

impl Index<usize> for Matrix {
    type Output = [f64];
    fn index(&self, index: usize) -> &Self::Output {
        &self.data[index * self.dimension..(index + 1) * self.dimension]
    }
}

impl IndexMut<usize> for Matrix {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.data[index * self.dimension..(index + 1) * self.dimension]
    }
}

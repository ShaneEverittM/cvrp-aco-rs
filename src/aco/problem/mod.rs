use std::{
    fs::File,
    io::{BufRead, BufReader},
    ops::{Deref, DerefMut},
};

use anyhow::{anyhow, Result};
use regex::Regex;

pub use matrix::Matrix;

use super::utils::ParseKeyValueAt;

mod matrix;

#[derive(Debug)]
pub struct Problem {
    pub adjacency_matrix: Matrix,
    pub demands: Vec<usize>,
    pub capacity: usize,
}

impl Problem {
    pub fn try_from_vrp(vrp: File) -> Result<Self> {
        let buf_reader = BufReader::new(vrp);

        let lines: Vec<String> = buf_reader.lines().collect::<std::io::Result<_>>()?;

        let (_, dimension) = lines.parse_key_value_at::<String, usize>(3)?;

        let (_, capacity) = lines.parse_key_value_at::<String, usize>(5)?;

        let mut adjacency_matrix = Matrix::new(dimension);

        let coordinates: Vec<NodeCoordinate> = lines[7..7 + dimension]
            .iter()
            .map(|s| NodeCoordinate::try_from_line(s))
            .collect::<Result<_>>()?;

        for (i, a) in coordinates.iter().enumerate() {
            for (j, b) in coordinates.iter().enumerate() {
                adjacency_matrix[i][j] = a.distance_from(b);
            }
        }

        let demands: Vec<usize> = lines[8 + dimension..8 + dimension * 2]
            .iter()
            .map(|s| Demand::try_from_line(s))
            .collect::<Result<Vec<Demand>>>()?
            .iter()
            .map(|d| d.0)
            .collect();

        Ok(Self {
            adjacency_matrix,
            demands,
            capacity,
        })
    }
}

#[derive(Default)]
struct NodeCoordinate {
    x: f64,
    y: f64,
}

impl NodeCoordinate {
    pub fn try_from_line(line: &str) -> Result<Self> {
        let regex = Regex::new(r"\s*[0-9.]+\s*(?P<x>[0-9.-]+)\s*(?P<y>[0-9.-]+)\s*").unwrap();

        // Try to get capture groups
        let captures = regex
            .captures(line)
            .ok_or_else(|| anyhow!("Cannot match {}", line))?;

        // Parse
        let x = captures.name("x").unwrap().as_str().parse::<f64>()?;
        let y = captures.name("y").unwrap().as_str().parse::<f64>()?;

        Ok(Self { x, y })
    }

    pub fn distance_from(&self, other: &NodeCoordinate) -> f64 {
        ((other.y - self.y).powi(2) + (other.x - self.x).powi(2)).sqrt()
    }
}

struct Demand(usize);

impl Demand {
    pub fn try_from_line(line: &str) -> Result<Self> {
        let regex = Regex::new(r"\s*[0-9.]+\s*(?P<demand>[0-9.]+)\s*").unwrap();

        let captures = regex
            .captures(line)
            .ok_or_else(|| anyhow!("Cannot match group"))?;

        Ok(Self(captures.name("demand").unwrap().as_str().parse()?))
    }
}

impl Deref for Demand {
    type Target = usize;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Demand {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

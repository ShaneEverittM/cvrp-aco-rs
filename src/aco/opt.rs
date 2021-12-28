use super::problem::Matrix;

pub trait OptimizationStrategy {
    fn convert_to_multiple_paths(path: &[usize]) -> Vec<Vec<usize>> {
        let mut paths = crate::aco::utils::path_to_routes(path);

        for path in &mut paths {
            path.push(0);
            path.insert(0, 0);
        }

        paths.remove(paths.len() - 1);

        paths
    }

    fn convert_to_single_path(paths: Vec<Vec<usize>>) -> Vec<usize> {
        paths.into_iter().flatten().collect()
    }

    fn calc_path_length(path: &[usize], adjacency_matrix: &Matrix) -> f64 {
        path.iter()
            .zip(path.iter().skip(1))
            .fold(0.0, |a, (&i, &j)| a + adjacency_matrix[i][j])
    }

    fn optimize(&self, path: &[usize], adjacency_matrix: &Matrix) -> (Vec<usize>, f64);
}

pub struct TwoOptStrategy;

impl TwoOptStrategy {
    fn optimize_path(path: Vec<usize>, adjacency_matrix: &Matrix) -> Vec<usize> {
        for i in 0..path.len() - 2 {
            for k in i + 1..path.len() - 1 {
                let removed_edge_cost =
                    adjacency_matrix[path[i]][path[i + 1]] + adjacency_matrix[path[k]][path[k + 1]];
                let new_edges_cost =
                    adjacency_matrix[path[i]][path[k]] + adjacency_matrix[path[i + 1]][path[k + 1]];

                if removed_edge_cost - new_edges_cost > 1.0 {
                    return Self::optimize_path(Self::swap(&path, i, k), adjacency_matrix);
                }
            }
        }
        path
    }

    fn swap(path: &[usize], i: usize, k: usize) -> Vec<usize> {
        let mut swapped_path: Vec<usize> = Vec::with_capacity(path.len());

        path[0..=i]
            .iter()
            .cloned()
            .for_each(|i| swapped_path.push(i));

        path[i + 1..=k]
            .iter()
            .cloned()
            .rev()
            .for_each(|i| swapped_path.push(i));

        path[k + 1..path.len()]
            .iter()
            .cloned()
            .for_each(|i| swapped_path.push(i));

        swapped_path
    }
}

impl OptimizationStrategy for TwoOptStrategy {
    fn optimize(&self, path: &[usize], adjacency_matrix: &Matrix) -> (Vec<usize>, f64) {
        let paths = Self::convert_to_multiple_paths(path);

        let new_path = Self::convert_to_single_path(
            paths
                .into_iter()
                .map(|p| Self::optimize_path(p, adjacency_matrix))
                .collect(),
        );

        // TODO: Can we calculate this while reordering?
        let length = Self::calc_path_length(&new_path, adjacency_matrix);

        (new_path, length)
    }
}

pub struct NoOpStrategy;

impl OptimizationStrategy for NoOpStrategy {
    fn optimize(&self, path: &[usize], adjacency_matrix: &Matrix) -> (Vec<usize>, f64) {
        (
            path.to_vec(),
            Self::calc_path_length(path, adjacency_matrix),
        )
    }
}

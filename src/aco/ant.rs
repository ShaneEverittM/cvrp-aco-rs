use rand::random;

use super::opt::OptimizationStrategy;

use yoos::collections::Matrix;

#[derive(Clone)]
pub struct Ant {
    path_taken: Vec<usize>,
    path_cost: f64,
    visited: Vec<bool>,
    visited_count: usize,
    num_nodes: usize,
    capacity: usize,
    cur_capacity: usize,
}

impl Ant {
    pub fn new(num_nodes: usize, capacity: usize) -> Self {
        let mut visited = vec![false; num_nodes];
        visited[0] = true;
        Self {
            path_taken: [0].into(),
            path_cost: 0.0,
            visited,
            visited_count: 1,
            num_nodes,
            capacity,
            cur_capacity: capacity,
        }
    }

    pub fn done(&self) -> bool {
        self.num_nodes == self.visited_count
    }

    pub fn move_to_next(
        &mut self,
        adjacency_matrix: &Matrix,
        pheromones: &Matrix,
        nodes: &[usize],
    ) {
        let cur_node = self.cur_node();

        if cur_node == 0 {
            self.cur_capacity = self.capacity;
        }

        let next_node = self.find_next_node(adjacency_matrix, pheromones, nodes);

        self.path_cost += adjacency_matrix[cur_node][next_node];

        self.cur_capacity -= *nodes.get(next_node).unwrap();

        self.visit(next_node);
    }

    pub fn cur_node(&self) -> usize {
        *self.path_taken.last().unwrap()
    }

    fn find_next_node(
        &self,
        adjacency_matrix: &Matrix,
        pheromones: &Matrix,
        nodes: &[usize],
    ) -> usize {
        let mut distribution_vec: Vec<Option<f64>> = vec![None; adjacency_matrix.size()];
        let cur_node = self.cur_node();
        let mut total_edge_weight: f64 = 0.0;

        for (i, d) in distribution_vec.iter_mut().enumerate() {
            if !self.visited[i] {
                let distance_to_depot = adjacency_matrix[cur_node][0];
                let distance_from_depot = adjacency_matrix[0][i];
                let distance_to_next = adjacency_matrix[cur_node][i];
                let savings = distance_to_depot + distance_from_depot - distance_to_next;

                let pheromone = pheromones[cur_node][i];
                let edge_weight = Self::calc_edge_weight(savings, pheromone, distance_to_next);
                total_edge_weight += edge_weight;
                *d = Some(edge_weight);
            }
        }

        let mut next_node =
            Self::get_next_node_by_probability(&distribution_vec, total_edge_weight);

        if self.cur_capacity < *nodes.get(next_node).unwrap() {
            next_node = 0;
        }

        next_node
    }

    fn get_next_node_by_probability(distribution: &[Option<f64>], total_edge_weight: f64) -> usize {
        let rand: f64 = random();
        let ratio: f64 = 1.0f64 / total_edge_weight;
        let mut temp_dist: f64 = 0.0;
        for (i, d) in distribution.iter().enumerate() {
            if let Some(d_value) = d {
                temp_dist += d_value;
                if rand / ratio <= temp_dist {
                    return i;
                }
            };
        }

        // TODO: Think about this
        unreachable!()
    }

    fn calc_edge_weight(savings: f64, pheromone: f64, distance_to_next: f64) -> f64 {
        let e = savings.powi(9);
        let p = pheromone.powi(2);
        let d = (1.0f64 / distance_to_next).powi(5);
        e * p * d
    }

    fn visit(&mut self, idx: usize) {
        self.path_taken.push(idx);
        // Don't mark twice
        if !self.visited[idx] {
            self.visited[idx] = true;
            self.visited_count += 1;
        }
    }

    pub fn path_cost(&self) -> f64 {
        self.path_cost
    }

    pub fn path_taken(&self) -> &Vec<usize> {
        &self.path_taken
    }

    pub fn complete(&mut self, adjacency_matrix: &Matrix) {
        self.path_cost += adjacency_matrix[self.cur_node()][0];
        self.visit(0);
    }

    pub fn optimize_path<S: OptimizationStrategy>(
        &mut self,
        adjacency_matrix: &Matrix,
        strategy: S,
    ) {
        let (path, cost) = strategy.optimize(self.path_taken(), adjacency_matrix);
        self.path_taken = path;
        self.path_cost = cost
    }
}

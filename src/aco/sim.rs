use super::{
    ant::Ant,
    opt::TwoOptStrategy,
    problem::{Problem},
};

use yoos::collections::Matrix;

const MAX_CYCLES: usize = 150;
const BEST_TOUR_COST: f64 = f64::MAX;

#[derive(Eq, PartialEq)]
enum Continue {
    Yes,
    No,
}

pub struct Simulator {
    //  Problem description
    adjacency_matrix: Matrix,
    demands: Vec<usize>,
    capacity: usize,

    // Ant tracking
    ants: Vec<Ant>,
    pheromones: Matrix,

    // Time out after this hits MAX_CYCLES
    cur_cycle: usize,
    cycles_since_improvement: usize,

    // Best path tracking
    best_tour_cost: f64,
    best_tour: Vec<usize>,
}

impl Simulator {
    pub fn on(problem: Problem) -> Self {
        let num_nodes = problem.adjacency_matrix.size();
        Self {
            adjacency_matrix: problem.adjacency_matrix,
            demands: problem.demands,
            capacity: problem.capacity,
            ants: Self::init_ants(num_nodes, num_nodes, problem.capacity),
            pheromones: Self::init_pheromones(num_nodes),
            cur_cycle: 0,
            cycles_since_improvement: 0,
            best_tour_cost: BEST_TOUR_COST,
            best_tour: Vec::new(),
        }
    }

    fn init_ants(num_nodes: usize, num_ants: usize, capacity: usize) -> Vec<Ant> {
        vec![Ant::new(num_nodes, capacity); num_ants]
    }

    fn init_pheromones(n: usize) -> Matrix {
        let mut pheromones = Matrix::new(n);
        for i in 0..n {
            for j in i + 1..n {
                pheromones[i][j] = 1.0;
                pheromones[j][i] = 1.0;
            }
        }
        pheromones
    }

    pub fn run(&mut self) -> anyhow::Result<()> {
        let now = std::time::Instant::now();
        while self.should_continue() {
            self.reset_ants();
            self.update_ants();
            if self.try_find_best_tour() == Continue::No {
                break;
            }
            self.evaporate();
            self.update_pheromones()
        }
        let time = now.elapsed();
        println!(
            "Best found VRP solutions of cost {} by visiting:",
            &self.best_tour_cost
        );
        println!("{}", Self::format_path(&self.best_tour));
        println!("Took {:?}", time);

        Ok(())
    }

    fn update_ants(&mut self) {
        for ant in &mut self.ants {
            while !ant.done() {
                ant.move_to_next(&self.adjacency_matrix, &self.pheromones, &self.demands);
            }
            ant.complete(&self.adjacency_matrix);
            ant.optimize_path(&self.adjacency_matrix, TwoOptStrategy)
        }
    }

    fn try_find_best_tour(&mut self) -> Continue {
        let mut found_better = false;
        for ant in self.ants.iter() {
            let tour_length = ant.path_cost();
            if self.best_tour_cost > tour_length {
                found_better = true;
                self.best_tour_cost = tour_length;
                self.best_tour = ant.path_taken().clone();
            }
        }

        if found_better {
            println!(
                "New best found VRP solution of cost {} visiting",
                self.best_tour_cost
            );
            println!("Current Paths:");
            println!("{}", Self::format_path(&self.best_tour));
            println!("Current cycle: {}", self.cur_cycle);
            self.cycles_since_improvement = 0;
            Continue::Yes
        } else {
            println!("Could not find route beating {}", self.best_tour_cost);
            println!("Current cycle: {}", self.cur_cycle);
            self.cycles_since_improvement += 1;
            if self.cycles_since_improvement > MAX_CYCLES / 2 {
                Continue::No
            } else {
                Continue::Yes
            }
        }
    }

    fn evaporate(&mut self) {
        let avg = self.ants.iter().map(Ant::path_cost).sum::<f64>() / self.ants.len() as f64;
        let evaporation_factor = 0.5 + 80.0 / avg;

        for i in 0..self.num_nodes() {
            for j in 0..self.num_nodes() {
                self.pheromones.update(i, j, |v| v * evaporation_factor);
                self.pheromones.update(j, i, |v| v * evaporation_factor);
            }
        }
    }

    fn update_pheromones(&mut self) {
        self.ants.sort_unstable_by(|ant1, ant2| {
            ant1.path_cost().partial_cmp(&ant2.path_cost()).unwrap()
        });

        let star_ant = self.ants.first().unwrap();
        let star_path = star_ant.path_taken();
        for i in 0..(star_path.len() - 1) {
            let pheromone = 3.0 / star_ant.path_cost();
            let u = *star_path.get(i).unwrap();
            let v = *star_path.get(i + 1).unwrap();
            self.pheromones.update(u, v, |v| v + pheromone);
        }

        for lambda in 1..3 {
            let cur_ant = self.ants.get(lambda).unwrap();
            let pheromone = (3 - lambda) as f64 / cur_ant.path_cost();

            let path_taken = cur_ant.path_taken();

            for i in 0..(path_taken.len() - 1) {
                let u = *path_taken.get(i).unwrap();
                let v = *path_taken.get(i + 1).unwrap();

                self.pheromones.update(u, v, |v| v + pheromone);
            }
        }
    }

    fn num_nodes(&self) -> usize {
        self.adjacency_matrix.size()
    }

    fn reset_ants(&mut self) {
        self.ants = vec![Ant::new(self.num_nodes(), self.capacity); self.num_nodes()];
    }

    fn should_continue(&mut self) -> bool {
        self.cur_cycle += 1;
        self.cur_cycle < MAX_CYCLES
    }

    fn format_path(path: &[usize]) -> String {
        let paths = crate::aco::utils::path_to_routes(path);

        let mut lines: Vec<String> = Vec::new();

        for (i, path) in paths.iter().enumerate() {
            if path.is_empty() {
                continue;
            };
            let line: String = path.iter().map(|&p| p.to_string() + " ").collect();
            lines.push(String::from("Route #") + &(i + 1).to_string() + ": " + &line);
        }

        lines.join("\n")
    }
}

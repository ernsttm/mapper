mod error;

use std::error::Error;
use std::fmt::{Debug, Formatter};
use std::fs::File;
use std::io::{BufRead, BufReader};

use error::PlacerError;

pub struct Config {
    filename: String,
}

impl Config {
    pub fn new(args: &[String]) -> Result<Config, PlacerError> {
        if 2 != args.len() {
            return Err(PlacerError { why: "Invalid number of arguments, expected 2." });
        }

        let filename = args[1].clone();
        Ok(Config { filename })
    }

    pub fn filename(&self) -> &String {
        &self.filename
    }
}

struct Coordinate {
    x: i32,
    y: i32,
}

impl Debug for Coordinate {
    fn fmt(&self, f: &mut Formatter) -> Result<(), std::fmt::Error> {
        writeln!(f, "[{}, {}]", self.x, self.y)
    }
}

struct Edge {
    node_a: usize,
    node_b: usize,
}

struct Problem {
    solve_diff: f64,
    floating_cells: usize,
    static_cells: Vec<Coordinate>,
    edges: Vec<Edge>,
}

struct Matrix {
    size: usize,
    values: Vec<Vec<i32>>
}

impl Debug for Matrix {
    fn fmt(&self, f: &mut Formatter) -> Result<(), std::fmt::Error> {
        for row in &self.values {
            writeln!(f, "{:?}", row)?;
        }

        Ok(())
    }
}

impl Matrix {
    fn new(size: usize) -> Matrix {
        Matrix { size, values: vec![vec![0; size]; size] }
    }

    fn add_node_view(&mut self, node: usize) -> () {
        self.values[node][node] += 2;
    }

    fn add_edge_view(&mut self, node_a: usize, node_b: usize) ->() {
        self.add_node_view(node_a);
        self.add_node_view(node_b);
        self.values[node_a][node_b] += -2;
        self.values[node_b][node_a] += -2;
    }

    fn solve(&self, solve_diff: f64, b: &Vec<i32>) -> Vec<i32> {
        let mut solution = vec![0.0; b.len()];

        loop {
            let mut iter_diff = 0.0;
            let mut new_solution_row = Vec::new();
            for row in 0..self.size {
                let mut new_solution = b[row] as f64;
                for column in 0..self.size {
                    // Skip the solver if it is the current equation.
                    if row == column {
                        continue;
                    }

                    new_solution -= self.values[row][column] as f64 * solution[column]
                }
                new_solution /= self.values[row][row] as f64;

                let diff = (solution[row] - new_solution).abs();
                if diff > iter_diff {
                    iter_diff = diff;
                }

                new_solution_row.push(new_solution);
            }
            solution = new_solution_row;

            if iter_diff < solve_diff {
                break;
            }
        }

        let mut rounded_solution = Vec::new();
        for value in solution {
            rounded_solution.push(value.round() as i32);
        }
        rounded_solution
    }
}

fn read_expected_line(reader: &mut BufReader<&File>) -> Result<String, Box<dyn Error>> {
    let mut line = String::new();
    let num_bytes = reader.read_line(&mut line)?;
    if 0 == num_bytes {
        return Err(PlacerError { why: "File contains no more lines" }.into());
    }

    line.pop();
    Ok(line)
}

fn read_int_array(reader: &mut BufReader<&File>) -> Result<Vec<i32>, Box<dyn Error>> {
    let line = read_expected_line(reader)?;
    let mut values = Vec::new();
    for value in line.split_whitespace() {
        values.push(value.parse()?);
    }

    Ok(values)
}

fn read_index_array(reader: &mut BufReader<&File>) -> Result<Vec<usize>, Box<dyn Error>> {
    let line = read_expected_line(reader)?;
    let mut values = Vec::new();
    for value in line.split_whitespace() {
        values.push(value.parse()?);
    }

    Ok(values)
}

fn parse_static_cells(reader: &mut BufReader<&File>, num_cells: usize)
    -> Result<Vec<Coordinate>, Box<dyn Error>> {
    let mut static_cells = Vec::new();
    for x in 0..num_cells {
        let static_cell = read_int_array(reader)?;
        if 2 != static_cell.len() {
            return Err(PlacerError { why: "Invalid gate definition" }.into());
        }

        static_cells.push(Coordinate { x: static_cell[0], y: static_cell[1] });
    }

    Ok(static_cells)
}

fn parse(config: &Config) -> Result<(Problem), Box<dyn Error>> {
    let file = File::open(config.filename())?;
    let mut reader = BufReader::new(&file);

    let solve_diff: f64 = read_expected_line(&mut reader)?.parse()?;
    let chip_info = read_int_array(&mut reader)?;

    // Load the static cells
    let num_statics = chip_info[0] as usize;
    let static_cells = parse_static_cells(&mut reader, num_statics)?;

    let mut edges = Vec::new();
    for x in 0..chip_info[2] {
        let edge = read_index_array(&mut reader)?;
        if 2 != edge.len() {
            return Err(PlacerError { why: "Invalid edge definition" }.into());
        }

        edges.push(Edge { node_a: edge[0], node_b: edge[1] });
    }

    Ok(Problem { solve_diff, floating_cells: chip_info[1] as usize, static_cells, edges })
}

fn solve_placement(prob: &Problem) -> Vec<Coordinate> {
    let num_statics = prob.static_cells.len();

    let mut a = Matrix::new(prob.floating_cells);
    let mut xb = vec![0; prob.floating_cells];
    let mut yb = vec![0; prob.floating_cells];
    for edge in &prob.edges {
        if edge.node_a < num_statics && edge.node_b >= num_statics {
            let static_node = edge.node_a;
            let floating_node = edge.node_b - num_statics;
            xb[floating_node] += prob.static_cells[static_node].x * 2;
            yb[floating_node] += prob.static_cells[static_node].y * 2;
            a.add_node_view(floating_node);
        } else if edge.node_a >= num_statics && edge.node_b < num_statics {
            let static_node = edge.node_b;
            let floating_node = edge.node_a - num_statics;
            xb[floating_node] += prob.static_cells[static_node].x * 2;
            yb[floating_node] += prob.static_cells[static_node].y * 2;
            a.add_node_view(floating_node)
        } else if edge.node_a >= num_statics && edge.node_b >= num_statics {
            let node_a = edge.node_a - num_statics;
            let node_b = edge.node_b - num_statics;
            a.add_edge_view(node_a, node_b);
        }
    }

    let x_results = a.solve(prob.solve_diff, &xb);
    let y_results = a.solve(prob.solve_diff, &yb);

    let mut solved_coordinates = Vec::new();
    for index in 0..prob.floating_cells {
        solved_coordinates.push(Coordinate { x: x_results[index], y: y_results[index] });
    }

    solved_coordinates
}

fn calculate_manhattan(edges: &Vec<Edge>, static_cells: &Vec<Coordinate>,
                       floating_cells: &Vec<Coordinate>) -> usize {
    let mut length= 0;
    for edge in edges {
        let coordinate_a;
        let coordinate_b;

        if edge.node_a < static_cells.len() {
            coordinate_a = &static_cells[edge.node_a];
        } else {
            coordinate_a = &floating_cells[edge.node_a - static_cells.len()];
        }

        if edge.node_b < static_cells.len() {
            coordinate_b = &static_cells[edge.node_b];
        } else {
            coordinate_b = &floating_cells[edge.node_b - static_cells.len()];
        }

        length += (coordinate_a.x - coordinate_b.x).abs();
        length += (coordinate_a.y - coordinate_b.y).abs();
    }

    length as usize
}

pub fn run(config: &Config) -> Result<(usize), Box<dyn Error>> {
    let prob = parse(config)?;
    let floating_cells = solve_placement(&prob);

    let offset = prob.static_cells.len();
    let mut count = 0;
    for cell in floating_cells.iter() {
        count += 1;
    }
    let manhattan = calculate_manhattan(&prob.edges, &prob.static_cells, &floating_cells);

    Ok(manhattan)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_input() {
        let args = [String::from("test1"), String::from("test2")];
        let config = Config::new(&args).unwrap();

        assert_eq!("test2", config.filename());
    }

    #[test]
    #[should_panic(expected = r#"Placer Error: Invalid number of arguments, expected 2."#)]
    fn too_few_arguments() {
        let args = [String::from("test1")];
        Config::new(&args).unwrap();
    }

    #[test]
    fn test1() {
        let args = [String::from("dummy_exe"), String::from("test1")];
        let config = Config::new(&args).unwrap();
        let length = run(&config).unwrap();

        assert_eq!(12, length);
    }

    #[test]
    fn test2() {
        let args = [String::from("dummy_exe"), String::from("test2")];
        let config = Config::new(&args).unwrap();
        let length = run(&config).unwrap();

        assert_eq!(12, length);
    }

    #[test]
    fn test3() {
        let args = [String::from("dummy_exe"), String::from("test3")];
        let config = Config::new(&args).unwrap();
        let length = run(&config).unwrap();

        assert_eq!(12, length);
    }

    #[test]
    fn test4() {
        let args = [String::from("dummy_exe"), String::from("test4")];
        let config = Config::new(&args).unwrap();
        let length = run(&config).unwrap();

        assert_eq!(42517, length);
    }

    #[test]
    #[ignore]
    fn test5() {
        let args = [String::from("dummy_exe"), String::from("test5")];
        let config = Config::new(&args).unwrap();
        let length = run(&config).unwrap();

        assert_eq!(833829, length);
    }
}

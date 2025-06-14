use crate::ltl_engine::config::Config;
use crate::ltl_engine::neighbourhood::Neighbourhood;
use rand::{Rng, RngCore};
use std::cmp::{max, min};
use std::ops::Range;
use std::thread;

pub type Cell = u8;
type Cells = Vec<Cell>;
type DoGetNeighbourhood = dyn Fn(&Board, usize, usize, usize, usize) -> Option<u8>;

/// A struct representing a game board of cells. Each cell can be in one of several states.
///
#[derive(Debug, Clone, PartialEq)]
pub struct Board {
    size: usize,
    chunks: Vec<Range<usize>>,
    config: Config,
    pub cells: Cells,
}

impl Board {
    /// Create a new board of cells with the given size and configuration.
    ///
    /// # Arguments
    ///
    /// * `size` - The size of the board. The board will be `size` by `size`.
    /// * `config` - The configuration of the board.
    ///
    pub fn new(size: u64, config: Config) -> Self {
        let cores = thread::available_parallelism().unwrap().get();
        Board {
            size: size as usize,
            chunks: Board::get_chunks(size as usize, cores),
            config,
            cells: vec![0; (size * size) as usize],
        }
    }

    /// Create a new board of cells with the given size, configuration and array of cells.
    ///
    /// # Arguments
    ///
    /// * `size` - The size of the board. The board will be `size` by `size`.
    /// * `config` - The configuration of the board.
    /// * `cells` - The array of cells.
    ///
    fn from_cells(config: Config, cells: Cells) -> Self {
        let cores = thread::available_parallelism().unwrap().get();
        let sq_root = (cells.len() as f32).sqrt();
        if sq_root.fract() != 0. {
            panic!("Only square matrix supported. Square root of length of cells is not an integer value.")
        }
        let size = sq_root as usize;
        Board {
            size,
            chunks: Board::get_chunks(size, cores),
            config,
            cells,
        }
    }

    /// Creates chunks of board optimized for parallel computing
    ///
    /// # Returns
    ///
    /// Vector of chunks.
    ///
    fn get_chunks(size: usize, cores: usize) -> Vec<Range<usize>> {
        let len = size * size;
        if len < cores {
            let mut chunks = Vec::with_capacity(len);
            for start in 0..len {
                chunks.push(start..(start + 1));
            }
            return chunks;
        }

        let chunk_size = len as f32 / cores as f32;
        let chunk_size_max = chunk_size.ceil() as usize;
        let chunk_size_min = chunk_size.floor() as usize;
        let mut chunks = Vec::with_capacity(cores);

        let mut cores_rem = cores;
        let mut start = 0;
        while cores_rem > 0 && (len - start) % cores_rem != 0 {
            let end = start + chunk_size_max;
            chunks.push(start..end);
            start = end;
            cores_rem -= 1;
        }
        for _ in 0..cores_rem {
            let end = start + chunk_size_min;
            chunks.push(start..end);
            start = end;
        }
        chunks
    }

    /// Reset all cells in the board to their initial state (0).
    ///
    pub fn reset(&mut self) -> () {
        for i in self.cells.iter_mut() {
            *i = 0;
        }
    }

    /// Translate the index of row and column into vector index
    ///
    /// # Arguments
    ///
    /// * `x` - The x coordinate of the cell.
    /// * `y` - The y coordinate of the cell.
    ///
    /// # Returns
    ///
    /// An index of the cell in vector
    fn get_index(&self, x: usize, y: usize) -> usize {
        return x * self.size as usize + y;
    }

    /// Get the value of the cell at the given coordinates.
    ///
    /// # Arguments
    ///
    /// * `x` - The x coordinate of the cell.
    /// * `y` - The y coordinate of the cell.
    ///
    /// # Returns
    ///
    /// The value of the cell at the given coordinates.
    ///
    pub fn get_cell(&self, x: usize, y: usize) -> Cell {
        self.cells[self.get_index(x, y)]
    }

    /// Set the value of the cell at the given coordinates.
    ///
    /// # Arguments
    ///
    /// * `x` - The x coordinate of the cell.
    /// * `y` - The y coordinate of the cell.
    /// * `value` - The value to set the cell to.
    ///
    /// # Panics
    ///
    /// This function will panic if `value` is greater than or equal to the maximum
    /// number of cell states specified in the board's configuration.
    ///
    pub fn set_cell(&mut self, x: usize, y: usize, value: Cell) -> () {
        if value >= max(self.config.cc, 2) {
            panic!("Config doesnt support provided value");
        }
        let i = self.get_index(x, y);
        self.cells[i] = value;
    }

    /// Randomize the values of all cells in the board.
    ///
    /// # Arguments
    ///
    /// * `rng` - A mutable reference to a random number generator implementing the RngCore trait.
    ///
    pub fn randomize<T: RngCore>(&mut self, rng: &mut T) -> () {
        // match seed {
        //     None => {
        //         let mut rng = rand::thread_rng();
        //         self._randomize(&mut rng)
        //     }
        //     Some(seed_num) => {
        //         let mut rng = Pcg32::seed_from_u64(seed_num);
        //         self._randomize(&mut rng)
        //     }
        // }
        for i in self.cells.iter_mut() {
            *i = rng.gen_range(0..max(self.config.cc, 2));
        }
    }

    /// Increment the state of the cell at the given coordinates by one.
    ///
    /// # Arguments
    ///
    /// * `x` - The x coordinate of the cell.
    /// * `y` - The y coordinate of the cell.
    ///
    pub fn cell_up(&mut self, x: usize, y: usize) -> () {
        let value = (self.get_cell(x, y) + 1) % max(self.config.cc, 2);
        self.set_cell(x, y, value);
    }

    /// Decrements the state of the cell at the given coordinates by 1.
    ///
    /// # Arguments
    ///
    /// * `x` - The x coordinate of the cell.
    /// * `y` - The y coordinate of the cell.
    ///
    pub fn cell_down(&mut self, x: usize, y: usize) -> () {
        if self.get_cell(x, y) == 0 {
            return;
        }
        let value = (self.get_cell(x, y) - 1) % max(self.config.cc, 2);
        self.set_cell(x, y, value);
    }

    /// Updates the state of all cells on the board according to the rules of the game.
    ///
    pub fn update(&mut self) -> () {
        self.cells = self._update();
    }

    /// Updates the state of all cells on the board according to the rules of the game.
    ///
    /// # Returns
    ///
    /// Updated board.
    ///
    fn _update(&self) -> Cells {
        let mut results: Cells = Vec::new();
        thread::scope(|scope| {
            let mut handlers = Vec::new();
            for c in self.chunks.clone() {
                handlers.push(scope.spawn(move || {
                    let mut res: Cells = vec![0; c.len()];
                    for i in c {
                        let x = i / self.size;
                        let y = i % self.size;
                        dbg!(x, y);
                        res.push(self.update_cell(x, y));
                    }
                    res
                }));
            }
            for handler in handlers {
                results.append(handler.join().unwrap().as_mut())
            }
        });
        results
    }

    /// Return updated state of cell at the given coordinates.
    ///
    /// # Arguments
    ///
    /// * `x` - The x coordinate of the cell.
    /// * `y` - The y coordinate of the cell.
    ///
    /// # Returns
    ///
    /// Updated state of cell at the given coordinates.
    ///
    fn update_cell(&self, x: usize, y: usize) -> Cell {
        let neighbourhood_count = self.get_neighbourhood_count(x, y);
        let state = self.get_cell(x, y);
        match state {
            0 => self.check_birth(neighbourhood_count) as u8,
            1 => {
                if self.check_survival(neighbourhood_count) {
                    1
                } else if self.config.cc > 2 {
                    2
                } else {
                    0
                }
            }
            _ => (state + 1) % self.config.cc,
        }
    }

    /// Check if cell satisfies birth condition
    ///
    /// # Arguments
    ///
    /// * `neighbourhood_count` - represents the number of live neighbours.
    ///
    /// # Returns
    ///
    /// A boolean indicating whether the neighbourhood_count is within the configured birth range.
    ///
    #[inline]
    fn check_birth(&self, neighbourhood_count: u16) -> bool {
        self.config.bb.0 <= neighbourhood_count && neighbourhood_count <= self.config.bb.1
    }

    /// Check if cell satisfies survival condition
    ///
    /// # Arguments
    ///
    /// * `neighbourhood_count` - represents the number of live neighbours.
    ///
    /// # Returns
    ///
    /// A boolean indicating whether the neighbourhood_count is within the configured survival range.
    ///
    #[inline]
    fn check_survival(&self, neighbourhood_count: u16) -> bool {
        self.config.ss.0 <= neighbourhood_count && neighbourhood_count <= self.config.ss.1
    }

    /// Returns the number of live cells in the neighbourhood of the cell at the given coordinates.
    ///
    /// # Arguments
    ///
    /// * `x` - The x coordinate of the cell.
    /// * `y` - The y coordinate of the cell.
    ///
    /// # Returns
    ///
    /// The number of live cells in the neighbourhood of the cell
    ///
    fn get_neighbourhood_count(&self, x: usize, y: usize) -> u16 {
        let neighbourhood = match self.config.nn {
            Neighbourhood::Neumann => self.get_neighbourhood_neumann(x, y),
            Neighbourhood::Moore => self.get_neighbourhood_moore(x, y),
        };
        neighbourhood
            .into_iter()
            .filter(|&s| s > 0)
            .collect::<Vec<Cell>>()
            .len() as u16
    }

    /// Returns the neighbourhood of the cell at the given coordinates.
    ///
    /// # Arguments
    ///
    /// * `x` - The x coordinate of the cell.
    /// * `y` - The y coordinate of the cell.
    /// * `callback` - Callback function counting neighbourhood.
    ///
    /// # Returns
    ///
    /// The neighbourhood of the cell at the given coordinates
    ///
    fn get_neighbourhood(&self, x: usize, y: usize, callback: &DoGetNeighbourhood) -> Vec<u8> {
        let min_bound = 0;
        let r = self.config.rr as usize;
        let max_bound = self.cells.len() - 1;

        let x_min = if r > x { min_bound } else { x - r };
        let y_min = if r > y { min_bound } else { y - r };

        let x_max = min(x + r, max_bound);
        let y_max = min(y + r, max_bound);

        let mut neighbourhood: Vec<Cell> = Vec::new();
        for x_i in x_min..=x_max {
            for y_i in y_min..=y_max {
                let cell = callback(&self, x, y, x_i, y_i);
                if let Some(s) = cell {
                    neighbourhood.push(s)
                }
            }
        }
        neighbourhood
    }

    /// Returns the neighbourhood of the cell at the given coordinates, using the von Neumann neighbourhood.
    ///
    /// # Arguments
    ///
    /// * `x` - The x coordinate of the cell.
    /// * `y` - The y coordinate of the cell.
    ///
    /// # Returns
    ///
    /// The neighbourhood of the cell at the given coordinates
    ///
    fn get_neighbourhood_neumann(&self, x: usize, y: usize) -> Vec<u8> {
        self.get_neighbourhood(x, y, &|board, x, y, x_i, y_i| {
            if board.config.mm == 0 && x_i == x && y_i == y {
                return None;
            }
            if x_i.abs_diff(x) + y_i.abs_diff(y) <= board.config.rr as usize {
                return Option::from(board.get_cell(x_i, y_i));
            }
            None
        })
    }

    /// Returns the neighbourhood of the cell at the given coordinates, using the Moore neighbourhood.
    ///
    /// # Arguments
    ///
    /// * `x` - The x coordinate of the cell.
    /// * `y` - The y coordinate of the cell.
    ///
    /// # Returns
    ///
    /// The neighbourhood of the cell at the given coordinates
    ///
    fn get_neighbourhood_moore(&self, x: usize, y: usize) -> Vec<u8> {
        self.get_neighbourhood(x, y, &|board, x, y, x_i, y_i| {
            if board.config.mm == 0 && x_i == x && y_i == y {
                return None;
            }
            Option::from(board.get_cell(x_i, y_i))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::rngs::mock::StepRng;

    fn test_cells() -> Cells {
        vec![0, 1, 2, 3, 4, 5, 6, 7, 8]
    }

    fn test_cells_default(size: usize) -> Cells {
        vec![0; size * size]
    }

    fn test_config_moore_included() -> Config {
        Config {
            rr: 1,
            cc: 0,
            mm: 1,
            ss: (2, 3),
            bb: (3, 3),
            nn: Neighbourhood::Moore,
        }
    }

    fn test_config_neumann_included() -> Config {
        Config {
            rr: 1,
            cc: 0,
            mm: 1,
            ss: (2, 3),
            bb: (3, 3),
            nn: Neighbourhood::Neumann,
        }
    }

    fn test_config_conways() -> Config {
        Config {
            rr: 1,
            cc: 0,
            mm: 0,
            ss: (2, 3),
            bb: (3, 3),
            nn: Neighbourhood::Moore,
        }
    }

    #[test]
    fn get_chunks_1_core() {
        let size = 100;
        let chunks = Board::get_chunks(size, 1);
        dbg!(chunks.clone());
        assert_eq!(chunks, vec![0..size * size]);
    }

    #[test]
    fn get_chunks_2_cores() {
        let chunks = Board::get_chunks(100, 2);
        assert_eq!(chunks, vec![0..5000, 5000..10000]);
    }

    #[test]
    fn get_chunks_8_cores() {
        let chunks = Board::get_chunks(3, 8);
        assert_eq!(chunks, vec![0..2, 2..3, 3..4, 4..5, 5..6, 6..7, 7..8, 8..9]);
    }

    #[test]
    fn get_chunks_low_size_many_cores() {
        let chunks = Board::get_chunks(3, 70);
        assert_eq!(
            chunks,
            vec![0..1, 1..2, 2..3, 3..4, 4..5, 5..6, 6..7, 7..8, 8..9]
        );
    }

    #[test]
    fn board_new() {
        let board = Board::new(3, test_config_conways());
        let cells = test_cells_default(3);
        let config = test_config_conways();

        assert_eq!(*board.cells, cells);
        assert_eq!(board.config, config);
    }

    #[test]
    fn reset() {
        let mut board = Board::from_cells(test_config_conways(), test_cells());
        let cells = test_cells_default(3);
        board.reset();

        assert_eq!(*board.cells, cells);
    }

    #[test]
    fn get_cell() {
        let board = Board::from_cells(test_config_conways(), test_cells());
        assert_eq!(board.get_cell(0, 0), 0);
        assert_eq!(board.get_cell(1, 1), 4);
        assert_eq!(board.get_cell(1, 2), 5);
    }

    #[test]
    #[should_panic]
    fn get_cell_out_of_range() {
        let board = Board::from_cells(test_config_conways(), test_cells());
        board.get_cell(5, 5);
    }

    #[test]
    fn set_cell() {
        let mut board = Board::new(3, test_config_conways());
        board.set_cell(0, 0, 1);
        assert_eq!(board.get_cell(0, 0), 1);
        board.set_cell(1, 2, 1);
        assert_eq!(board.get_cell(1, 2), 1);
    }

    #[test]
    #[should_panic]
    fn set_cell_out_of_config_range() {
        let mut board = Board::new(3, test_config_conways());
        board.set_cell(0, 0, 2);
    }

    #[test]
    #[should_panic]
    fn set_cell_out_of_vector_range() {
        let mut board = Board::new(3, test_config_conways());
        board.set_cell(5, 0, 1);
    }

    #[test]
    fn randomize() {
        let mut board = Board::new(3, test_config_conways());
        let mut rnd = StepRng::new(u64::MAX, 0);
        board.randomize(&mut rnd);
        assert_eq!(*board.cells, vec![1; 9]);
    }

    #[test]
    fn randomize_multistate() {
        let mut board = Board::from_cells(
            Config {
                rr: 1,
                cc: 5,
                mm: 0,
                ss: (2, 3),
                bb: (3, 3),
                nn: Neighbourhood::Moore,
            },
            test_cells_default(3),
        );
        let mut rnd = StepRng::new(u64::MAX, 1);
        board.randomize(&mut rnd);
        let right = vec![4, 0, 0, 0, 0, 0, 0, 0, 0];
        assert_eq!(*board.cells, right);
    }

    #[test]
    fn cell_up() {
        let mut board = Board::new(3, test_config_conways());
        board.cell_up(0, 0);
        assert_eq!(board.get_cell(0, 0), 1);
        board.cell_up(0, 0);
        assert_eq!(board.get_cell(0, 0), 0);
    }

    #[test]
    fn cell_up_multistate() {
        let mut board = Board::from_cells(
            Config {
                rr: 1,
                cc: 3,
                mm: 0,
                ss: (2, 3),
                bb: (3, 3),
                nn: Neighbourhood::Moore,
            },
            test_cells_default(3),
        );
        board.cell_up(0, 0);
        assert_eq!(board.get_cell(0, 0), 1);
        board.cell_up(0, 0);
        assert_eq!(board.get_cell(0, 0), 2);
        board.cell_up(0, 0);
        assert_eq!(board.get_cell(0, 0), 0);
    }

    #[test]
    fn cell_down() {
        let mut board = Board::new(3, test_config_conways());
        board.set_cell(0, 0, 1);
        board.cell_down(0, 0);
        assert_eq!(board.get_cell(0, 0), 0);
    }

    #[test]
    fn cell_down_zero() {
        let mut board = Board::new(3, test_config_conways());
        board.cell_down(0, 0);
        assert_eq!(board.get_cell(0, 0), 0);
    }

    #[test]
    fn cell_down_multistate() {
        let mut board = Board::from_cells(
            Config {
                rr: 1,
                cc: 3,
                mm: 0,
                ss: (2, 3),
                bb: (3, 3),
                nn: Neighbourhood::Moore,
            },
            test_cells_default(3),
        );

        board.set_cell(0, 0, 2);

        board.cell_down(0, 0);
        assert_eq!(board.get_cell(0, 0), 1);
        board.cell_down(0, 0);
        assert_eq!(board.get_cell(0, 0), 0);
    }

    #[test]
    fn get_neighbourhood_moore_with_center() {
        let config = Config {
            rr: 1,
            cc: 0,
            mm: 0,
            ss: (2, 3),
            bb: (3, 3),
            nn: Neighbourhood::Moore,
        };
        let cells = test_cells();
        let board: Board = Board::from_cells(config, cells);
        let mut neighbourhood = board.get_neighbourhood_moore(1, 1);
        assert_eq!(neighbourhood.len(), 8);

        let right: Vec<u8> = vec![0, 1, 2, 3, 5, 6, 7, 8];
        neighbourhood.sort();
        assert_eq!(neighbourhood[..], right[..]);
    }

    #[test]
    fn get_neighbourhood_moore_without_center() {
        let config = Config {
            rr: 1,
            cc: 0,
            mm: 1,
            ss: (2, 3),
            bb: (3, 3),
            nn: Neighbourhood::Moore,
        };
        let cells = test_cells();
        let board: Board = Board::from_cells(config, cells);
        let mut neighbourhood = board.get_neighbourhood_moore(1, 1);
        assert_eq!(neighbourhood.len(), 9);

        let mut right: Vec<u8> = vec![0, 1, 2, 3, 4, 5, 6, 7, 8];
        neighbourhood.sort();
        right.sort();
        assert_eq!(neighbourhood[..], right[..]);
    }

    #[test]
    fn get_neighbourhood_count_moore_without_center() {
        let config = Config {
            rr: 1,
            cc: 25,
            mm: 0,
            ss: (2, 3),
            bb: (3, 3),
            nn: Neighbourhood::Moore,
        };
        let board = Board::from_cells(config, vec![0, 0, 0, 1, 2, 3, 4, 0, 0]);
        let neighbourhood_count = board.get_neighbourhood_count(1, 1);
        assert_eq!(neighbourhood_count, 3);
    }

    #[test]
    fn get_neighbourhood_count_moore_empty() {
        let config = Config {
            rr: 1,
            cc: 25,
            mm: 0,
            ss: (2, 3),
            bb: (3, 3),
            nn: Neighbourhood::Moore,
        };
        let board = Board::from_cells(config, test_cells_default(3));
        let neighbourhood_count = board.get_neighbourhood_count(1, 1);
        assert_eq!(neighbourhood_count, 0);
    }

    #[test]
    fn get_neighbourhood_count_moore_full() {
        let config = Config {
            rr: 1,
            cc: 25,
            mm: 1,
            ss: (2, 3),
            bb: (3, 3),
            nn: Neighbourhood::Moore,
        };
        let board = Board::from_cells(config, vec![1, 2, 3, 4, 5, 6, 7, 8, 9]);
        let neighbourhood_count = board.get_neighbourhood_count(1, 1);
        assert_eq!(neighbourhood_count, 9);
    }

    #[test]
    fn get_neighbourhood_count_neumann_with_center() {
        let config = Config {
            rr: 1,
            cc: 25,
            mm: 1,
            ss: (2, 3),
            bb: (3, 3),
            nn: Neighbourhood::Neumann,
        };
        let board = Board::from_cells(config, vec![0, 5, 0, 1, 2, 3, 4, 0, 0]);
        let neighbourhood_count = board.get_neighbourhood_count(1, 1);
        assert_eq!(neighbourhood_count, 4);
    }

    #[test]
    fn get_neighbourhood_count_neumann_without_center() {
        let config = Config {
            rr: 1,
            cc: 25,
            mm: 0,
            ss: (2, 3),
            bb: (3, 3),
            nn: Neighbourhood::Neumann,
        };
        let board = Board::from_cells(config, vec![0, 5, 0, 1, 2, 3, 4, 0, 0]);
        let neighbourhood_count = board.get_neighbourhood_count(1, 1);
        assert_eq!(neighbourhood_count, 3);
    }

    #[test]
    fn get_neighbourhood_count_neumann_empty() {
        let config = Config {
            rr: 1,
            cc: 25,
            mm: 0,
            ss: (2, 3),
            bb: (3, 3),
            nn: Neighbourhood::Neumann,
        };
        let board = Board::from_cells(config, test_cells_default(3));
        let neighbourhood_count = board.get_neighbourhood_count(1, 1);
        assert_eq!(neighbourhood_count, 0);
    }

    #[test]
    fn get_neighbourhood_count_neumann_full() {
        let config = Config {
            rr: 1,
            cc: 25,
            mm: 1,
            ss: (2, 3),
            bb: (3, 3),
            nn: Neighbourhood::Neumann,
        };
        let board = Board::from_cells(config, vec![1, 2, 3, 4, 5, 6, 7, 8, 9]);
        let neighbourhood_count = board.get_neighbourhood_count(1, 1);
        assert_eq!(neighbourhood_count, 5);
    }

    #[test]
    fn update_all_alive() {
        let config = Config {
            rr: 1,
            cc: 1,
            mm: 0,
            ss: (2, 100),
            bb: (0, 2),
            nn: Neighbourhood::Moore,
        };
        let mut board: Board = Board::from_cells(config, vec![0; 100]);
        // board.cells = test_cells_default(10);
        let right = vec![1; 100];
        board.update();

        assert_eq!(board.cells, right);
    }

    #[test]
    fn update_all_dead() {
        let config = Config {
            rr: 1,
            cc: 1,
            mm: 0,
            ss: (100, 123),
            bb: (0, 2),
            nn: Neighbourhood::Moore,
        };
        let mut board: Board = Board::from_cells(config, vec![2; 100]);
        let right = vec![0; 100];
        board.update();

        assert_eq!(*board.cells, right);
    }

    #[test]
    fn update_aging() {
        let mut board = Board::from_cells(
            Config {
                rr: 1,
                cc: 3,
                mm: 0,
                ss: (2, 3),
                bb: (3, 3),
                nn: Neighbourhood::Moore,
            },
            vec![1; 9],
        );
        board.update();
        let right = vec![1, 2, 1, 2, 2, 2, 1, 2, 1];
        assert_eq!(*board.cells, right);
    }

    #[test]
    fn update_stick_repeat() {
        let config = test_config_conways();
        let mut board: Board = Board::new(3, config);
        let stick_vertical = vec![0, 0, 0, 1, 1, 1, 0, 0, 0];
        let stick_horizontal = vec![0, 1, 0, 0, 1, 0, 0, 1, 0];

        board.cells = stick_vertical.clone();
        board.update();
        assert_eq!(*board.cells.clone(), stick_horizontal.clone());
        board.update();
        assert_eq!(*board.cells, stick_vertical);
        board.update();
        assert_eq!(*board.cells, stick_horizontal);
    }

    #[test]
    fn get_neighbourhood_moore_left_top_corner() {
        let board: Board = Board::from_cells(test_config_moore_included(), test_cells());
        let mut neighbourhood = board.get_neighbourhood_moore(0, 0);
        assert_eq!(neighbourhood.len(), 4);

        let mut right: Vec<u8> = vec![0, 1, 3, 4];
        neighbourhood.sort();
        right.sort();
        assert_eq!(neighbourhood[..], right[..]);
    }

    #[test]
    fn get_neighbourhood_moore_right_top_corner() {
        let board: Board = Board::from_cells(test_config_moore_included(), test_cells());
        let mut neighbourhood = board.get_neighbourhood_moore(2, 0);
        assert_eq!(neighbourhood.len(), 4);

        let mut right: Vec<u8> = vec![3, 6, 4, 7];
        right.sort();
        neighbourhood.sort();
        assert_eq!(neighbourhood[..], right[..]);
    }

    #[test]
    fn get_neighbourhood_moore_left_bottom_corner() {
        let board: Board = Board::from_cells(test_config_moore_included(), test_cells());
        let mut neighbourhood = board.get_neighbourhood_moore(0, 2);
        assert_eq!(neighbourhood.len(), 4);

        let mut right: Vec<u8> = vec![1, 4, 2, 5];
        right.sort();
        neighbourhood.sort();
        assert_eq!(neighbourhood[..], right[..]);
    }

    #[test]
    fn get_neighbourhood_moore_right_bottom_corner() {
        let board: Board = Board::from_cells(test_config_moore_included(), test_cells());
        let mut neighbourhood = board.get_neighbourhood_moore(2, 2);
        assert_eq!(neighbourhood.len(), 4);

        let mut right: Vec<u8> = vec![4, 7, 5, 8];
        right.sort();
        neighbourhood.sort();
        assert_eq!(neighbourhood[..], right[..]);
    }

    #[test]
    fn get_neighbourhood_moore_bigger_radius() {
        let config = Config {
            rr: 2,
            cc: 1,
            mm: 1,
            ss: (2, 3),
            bb: (3, 3),
            nn: Neighbourhood::Moore,
        };
        let cells = (0..=49).collect();
        let board: Board = Board::from_cells(config, cells);
        let mut neighbourhood = board.get_neighbourhood_moore(3, 3);
        assert_eq!(neighbourhood.len(), 25);

        let mut right: Vec<u8> = vec![
            8, 9, 10, 11, 12, 15, 16, 17, 18, 19, 22, 23, 24, 25, 26, 29, 30, 31, 32, 33, 36, 37,
            38, 39, 40,
        ];
        right.sort();
        neighbourhood.sort();
        assert_eq!(neighbourhood[..], right[..]);
    }

    #[test]
    fn get_neighbourhood_neumann_with_center() {
        let config = Config {
            rr: 1,
            cc: 0,
            mm: 1,
            ss: (2, 3),
            bb: (3, 3),
            nn: Neighbourhood::Neumann,
        };
        let board: Board = Board::from_cells(config, test_cells());
        let mut neighbourhood = board.get_neighbourhood_neumann(1, 1);
        assert_eq!(neighbourhood.len(), 5);

        let mut right: Vec<u8> = vec![1, 3, 4, 5, 7];
        right.sort();
        neighbourhood.sort();
        assert_eq!(neighbourhood[..], right[..]);
    }

    #[test]
    fn get_neighbourhood_neumann_without_center() {
        let config = Config {
            rr: 1,
            cc: 0,
            mm: 0,
            ss: (2, 3),
            bb: (3, 3),
            nn: Neighbourhood::Neumann,
        };
        let board: Board = Board::from_cells(config, test_cells());
        let mut neighbourhood = board.get_neighbourhood_neumann(1, 1);
        assert_eq!(neighbourhood.len(), 4);

        let mut right: Vec<u8> = vec![1, 3, 5, 7];
        right.sort();
        neighbourhood.sort();
        assert_eq!(neighbourhood[..], right[..]);
    }

    #[test]
    fn get_neighbourhood_neumann_left_top_corner() {
        let board: Board = Board::from_cells(test_config_neumann_included(), test_cells());
        let mut neighbourhood = board.get_neighbourhood_neumann(0, 0);
        assert_eq!(neighbourhood.len(), 3);

        let mut right: Vec<u8> = vec![0, 1, 3];
        right.sort();
        neighbourhood.sort();
        assert_eq!(neighbourhood[..], right[..]);
    }

    #[test]
    fn get_neighbourhood_neumann_right_top_corner() {
        let board: Board = Board::from_cells(test_config_neumann_included(), test_cells());
        let mut neighbourhood = board.get_neighbourhood_neumann(2, 0);
        assert_eq!(neighbourhood.len(), 3);

        let mut right: Vec<u8> = vec![3, 6, 7];
        right.sort();
        neighbourhood.sort();
        assert_eq!(neighbourhood[..], right[..]);
    }

    #[test]
    fn get_neighbourhood_neumann_left_bottom_corner() {
        let board: Board = Board::from_cells(test_config_neumann_included(), test_cells());
        let mut neighbourhood = board.get_neighbourhood_neumann(0, 2);
        assert_eq!(neighbourhood.len(), 3);

        let mut right: Vec<u8> = vec![1, 2, 5];
        right.sort();
        neighbourhood.sort();
        assert_eq!(neighbourhood[..], right[..]);
    }

    #[test]
    fn get_neighbourhood_neumann_right_bottom_corner() {
        let board: Board = Board::from_cells(test_config_neumann_included(), test_cells());
        let mut neighbourhood = board.get_neighbourhood_neumann(2, 2);
        assert_eq!(neighbourhood.len(), 3);

        let mut right: Vec<u8> = vec![5, 7, 8];
        right.sort();
        neighbourhood.sort();
        assert_eq!(neighbourhood[..], right[..]);
    }

    #[test]
    fn get_neighbourhood_neumann_bigger_radius() {
        let config = Config {
            rr: 2,
            cc: 1,
            mm: 1,
            ss: (2, 3),
            bb: (3, 3),
            nn: Neighbourhood::Neumann,
        };
        let cells = (0..49).collect();
        let board: Board = Board::from_cells(config, cells);
        let mut neighbourhood = board.get_neighbourhood_neumann(3, 3);
        assert_eq!(neighbourhood.len(), 13);

        let mut right: Vec<u8> = vec![10, 16, 17, 18, 22, 23, 24, 25, 26, 30, 31, 32, 38];
        right.sort();
        neighbourhood.sort();
        assert_eq!(neighbourhood[..], right[..]);
    }

    #[test]
    fn get_neighbourhood_count_moore_with_center() {
        let config = Config {
            rr: 1,
            cc: 25,
            mm: 1,
            ss: (2, 3),
            bb: (3, 3),
            nn: Neighbourhood::Moore,
        };
        let board = Board::from_cells(config, vec![0, 0, 0, 1, 2, 3, 4, 0, 0]);
        let neighbourhood_count = board.get_neighbourhood_count(1, 1);
        assert_eq!(neighbourhood_count, 4);
    }
}

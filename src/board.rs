use crate::config::Config;
use wasm_bindgen::prelude::*;

use crate::ltl_engine::board;
use crate::rnd::Rng;

#[wasm_bindgen]
#[derive(Debug, PartialEq)]
pub struct Board {
    board: board::Board,
}

#[wasm_bindgen]
impl Board {
    #[wasm_bindgen(constructor)]
    pub fn new(size: u64, config: &Config) -> Board {
        Board {
            board: board::Board::new(size, config.config()),
        }
    }

    pub fn reset(&mut self) -> () {
        self.board.reset();
    }

    pub fn get_cell(&self, x: usize, y: usize) -> board::Cell {
        self.board.get_cell(x, y)
    }

    pub fn set_cell(&mut self, x: usize, y: usize, value: u8) -> () {
        self.board.set_cell(x, y, value);
    }

    pub fn randomize(&mut self) -> () {
        let mut r = Rng {};
        self.board.randomize(&mut r)
    }

    pub fn cell_up(&mut self, x: usize, y: usize) -> () {
        self.board.cell_up(x, y)
    }

    pub fn cell_down(&mut self, x: usize, y: usize) -> () {
        self.board.cell_down(x, y)
    }

    pub fn update(&mut self) -> () {
        self.board.update()
    }
}

use std::{time::Duration, vec};

use bevy::prelude::*;
use rand::{thread_rng, Rng};

pub const OPENED_INDEX: u32 = 0;
pub const UNOPENED_INDEX: u32 = 9;
pub const BOMB_INDEX: u32 = 10;
pub const BOMB_RED_INDEX: u32 = 11;
pub const MARKED_INDEX: u32 = 13;

#[derive(Resource)]
pub struct StartTime(pub Timer);

impl Default for StartTime {
    fn default() -> Self {
        Self(Timer::new(Duration::from_secs(999), TimerMode::Once))
    }
}

#[derive(Resource, Default)]
pub struct LastStep {
    pub coord: Option<(u32, u32)>,
    pub uncover: bool,
}

impl LastStep {
    pub fn reset(&mut self) {
        self.coord = None;
        self.uncover = false;
    }
}

#[derive(Resource)]
pub struct BoardOptions {
    pub width: u32,
    pub height: u32,
    pub bomb_count: u32,
}

impl Default for BoardOptions {
    fn default() -> Self {
        Self {
            width: 9,
            height: 9,
            bomb_count: 10,
        }
    }
}

impl BoardOptions {
    pub fn area(&self) -> u32 {
        self.width * self.height
    }
}

#[derive(Resource, Default)]
pub struct Board {
    pub map: Vec<Vec<u32>>,
}

const SQUARE_COORD: [(i8, i8); 8] = [
    // Top Left
    (-1, 1),
    // Top
    (0, 1),
    // Top right
    (1, 1),
    // Left
    (-1, 0),
    // Right
    (1, 0),
    // Bottom left
    (-1, -1),
    // Bottom
    (0, -1),
    // Bottom right
    (1, -1),
];

impl Board {
    pub fn reset(&mut self, options: &Res<BoardOptions>) {
        let mut rng = thread_rng();
        let mut map = (0..options.area())
            .map(|i| {
                if i < options.bomb_count {
                    BOMB_INDEX
                } else {
                    OPENED_INDEX
                }
            })
            .collect::<Vec<u32>>();

        for i in 0..options.area() {
            let index = i as usize;
            let random = rng.gen_range(0..options.area()) as usize;
            if map[index] != map[random] {
                map[index] = OPENED_INDEX;
                map[random] = BOMB_INDEX;
            }
        }

        self.map = map
            .chunks(options.width as usize)
            .map(|i| i.to_vec())
            .collect();

        self.console_output();
    }

    pub fn get(&self, coord: (u32, u32)) -> u32 {
        let x = coord.0 as usize;
        let y = coord.1 as usize;
        if let Some(x_val) = self.map.get(x) {
            if let Some(y_val) = x_val.get(y) {
                return *y_val;
            }
        }
        0
    }

    pub fn is_bomb_at(&self, coord: (u32, u32)) -> bool {
        if self.get(coord) == BOMB_INDEX {
            return true;
        }
        false
    }

    pub fn bomb_count_at(&self, coord: (u32, u32)) -> u32 {
        if self.is_bomb_at(coord) {
            return 0;
        }
        let res = self
            .safe_square_at(coord)
            .into_iter()
            .filter(|coord| self.is_bomb_at(*coord))
            .count();
        res as u32
    }

    pub fn safe_square_at(&self, coord: (u32, u32)) -> Vec<(u32, u32)> {
        SQUARE_COORD
            .iter()
            .copied()
            .map(|tuple| {
                (
                    (coord.0 as i32 + tuple.0 as i32),
                    (coord.1 as i32 + tuple.1 as i32),
                )
            })
            .filter(|coord| {
                coord.0 >= 0
                    && coord.1 >= 0
                    && coord.0 < (self.map.len() as i32)
                    && coord.1 < (self.map.len() as i32)
            })
            .map(|coord| (coord.0 as u32, coord.1 as u32))
            .collect()
    }

    pub fn console_output(&self) -> String {
        let separator: String = (0..=self.map.len() * 3).map(|_| '-').collect();
        let mut board = vec![];

        for i in 0..self.map.len() {
            let mut row = vec![];
            for j in 0..self.map.len() {
                let column = self.map[j][i];
                row.push(format!("{:2}", column));
            }
            board.push(format!("|{}|", row.join(" ")));
        }
        board.reverse();
        format!("{}\n{}\n{}", separator, board.join("\n"), separator)
    }
}

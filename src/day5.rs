use anyhow::{anyhow, Context, Result};
use std::{fmt, fs};

const POSITION_LEN: usize = 10;
const COL_LEN: usize = 3;
const COL_MASK: u16 = (1 << COL_LEN) - 1;
const ROW_LEN: usize = POSITION_LEN - COL_LEN;
const ROW_MASK: u16 = (1 << POSITION_LEN) - COL_MASK - 1;

#[derive(Copy, Clone)]
struct Position(u16);

impl Position {
    fn from_str(s: &str) -> Self {
        let mut pos = 0u16;
        for (idx, c) in s.chars().enumerate().take(POSITION_LEN) {
            let idx = POSITION_LEN - idx - 1;
            let bit = match c {
                'B' | 'R' => 1,
                _ => 0,
            };
            pos |= bit << idx;
        }
        Self(pos)
    }

    fn row(self) -> u16 {
        (self.0 & ROW_MASK) >> COL_LEN
    }

    fn col(self) -> u16 {
        self.0 & COL_MASK
    }

    fn seat_id(self) -> u16 {
        self.row() * 8 + self.col()
    }
}

impl fmt::Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let row = self.row();
        let col = self.col();

        let mut line = String::with_capacity(POSITION_LEN);

        for idx in (0..ROW_LEN).rev() {
            let bit = (row & (1 << idx)) >> idx;
            let c = if bit == 1 { 'B' } else { 'F' };
            line.push(c);
        }
        for idx in (0..COL_LEN).rev() {
            let bit = (col & (1 << idx)) >> idx;
            let c = if bit == 1 { 'R' } else { 'L' };
            line.push(c);
        }

        f.write_str(&line)
    }
}

impl fmt::Debug for Position {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Position")
            .field("row", &self.row())
            .field("col", &self.col())
            .field("seat_id", &self.seat_id())
            .field("str", &self.to_string())
            .finish()
    }
}

pub fn run(args: &[&str]) -> Result<()> {
    let input = fs::read_to_string(args[0]).context("Failed to read file")?;

    let mut seat_ids = input
        .lines()
        .map(Position::from_str)
        .map(Position::seat_id)
        .collect::<Vec<_>>();

    seat_ids.sort_unstable();

    let max_seat_id = seat_ids.last().ok_or_else(|| anyhow!("No seats"))?;
    dbg!(max_seat_id);

    let my_id = seat_ids
        .windows(2)
        .find(|ids| matches!(ids, [id1, id2] if *id1 != id2 - 1))
        .and_then(<[_]>::first)
        .map(|prev_id| prev_id + 1)
        .ok_or_else(|| anyhow!("Failed to find my seat id"))?;
    dbg!(my_id);

    Ok(())
}

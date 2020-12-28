#![allow(clippy::reversed_empty_ranges)]

use anyhow::{Context, Result};
use arrayvec::ArrayVec;
use fixedbitset::FixedBitSet;
use ndarray::{azip, s, Array, Array2};
use std::{
    collections::hash_map::DefaultHasher,
    fmt, fs,
    hash::{Hash, Hasher},
    iter::{self, FromIterator},
    mem, str,
};

#[derive(Debug)]
struct Layout {
    occupied: Array2<u8>,
    floor_mask: Array2<u8>,
    scratch: Array2<u8>,
}

impl Layout {
    fn from_str(input: &str) -> Self {
        let mut n: usize = 0;
        let mut m: usize = 0;
        let elem_iter = input.lines().flat_map(|line| {
            n += 1;
            m = line.len();
            line.chars().map(|c| match c {
                'L' => 1,
                '.' => 0,
                _ => panic!("unexpected char: {}", c),
            })
        });
        // include border of 0's
        let floor_mask = Array::from_iter(elem_iter).into_shape((n, m)).unwrap();

        // include border of 0's
        // initial layout is all empty
        let occupied = Array2::zeros(((n + 2), (m + 2)));
        let scratch = Array2::zeros((n, m));

        Self {
            occupied,
            floor_mask,
            scratch,
        }
    }

    fn nrows(&self) -> usize {
        self.floor_mask.nrows()
    }

    fn ncols(&self) -> usize {
        self.floor_mask.ncols()
    }

    fn hash(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.occupied.hash(&mut hasher);
        hasher.finish()
    }

    fn count_occupied(&self) -> u16 {
        let occupied = self.occupied.as_slice_memory_order().unwrap();
        occupied.iter().map(|&val| val as u16).sum()
    }

    fn step(&mut self) {
        let mut neigh = self.scratch.view_mut();
        neigh.fill(0);

        // di = -1, dj = {-1, 0, 1}
        neigh += &self.occupied.slice(s![0..-2, 0..-2]);
        neigh += &self.occupied.slice(s![0..-2, 1..-1]);
        neigh += &self.occupied.slice(s![0..-2, 2..]);

        // di = 0, dj = {-1, 1}
        neigh += &self.occupied.slice(s![1..-1, 0..-2]);
        neigh += &self.occupied.slice(s![1..-1, 2..]);

        // di = 1, dj = {-1, 0, 1}
        neigh += &self.occupied.slice(s![2.., 0..-2]);
        neigh += &self.occupied.slice(s![2.., 1..-1]);
        neigh += &self.occupied.slice(s![2.., 2..]);

        let mut occupied = self.occupied.slice_mut(s![1..-1, 1..-1]);

        // If a seat is empty and there are no occupied neighbors, it becomes occupied.
        // If a seat is occupied and that are 4 or more neighbors, it becomes unoccupied.
        occupied.zip_mut_with(&neigh, |o, &n| {
            *o = ((*o == 0 && n == 0) || (*o == 1 && n < 4)) as u8
        });

        // Floor tiles should remain floor tiles
        occupied *= &self.floor_mask;
    }
}

impl fmt::Display for Layout {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let nrows = self.nrows();
        let ncols = self.ncols();

        let mut str_buf = String::with_capacity(ncols + 1);

        let occupied_view = self.occupied.slice(s![1..-1, 1..-1]);
        azip!((index (i, j), &occupied in &occupied_view, &mask in &self.floor_mask) {
            if mask == 0 {
                str_buf.push('.');
            } else if occupied == 1 {
                str_buf.push('#');
            } else {
                str_buf.push('L');
            }

            if j == ncols - 1 {
                if i != nrows - 1 {
                    str_buf.push('\n');
                }
                let _ = f.write_str(&str_buf);
                str_buf.clear();
            }
        });

        Ok(())
    }
}

const DIRECTIONS: [(i8, i8); 8] = [
    (-1, -1),
    (-1, 0),
    (-1, 1),
    (0, -1),
    (0, 1),
    (1, -1),
    (1, 0),
    (1, 1),
];

struct Layout2 {
    nrows: usize,
    ncols: usize,
    occupied: FixedBitSet,
    floor_mask: FixedBitSet,
    scratch: FixedBitSet,
    neighbor_indices: Vec<ArrayVec<[usize; 8]>>,
}

impl Layout2 {
    #[inline]
    const fn conv_1d_to_2d(ncols: usize, idx_1d: usize) -> (usize, usize) {
        let row_idx = idx_1d / ncols;
        let col_idx = idx_1d % ncols;
        (row_idx, col_idx)
    }

    #[inline]
    const fn conv_2d_to_1d(ncols: usize, row_idx: usize, col_idx: usize) -> usize {
        row_idx * ncols + col_idx
    }

    fn indices_in_direction(
        nrows: usize,
        ncols: usize,
        row_idx: usize,
        col_idx: usize,
        dr: i8,
        dc: i8,
    ) -> impl Iterator<Item = (usize, usize)> {
        let nrows = nrows as isize;
        let ncols = ncols as isize;
        let dr = dr as isize;
        let dc = dc as isize;
        iter::successors(Some((row_idx, col_idx)), move |(row_idx, col_idx)| {
            let new_row_idx = *row_idx as isize + dr;
            let new_col_idx = *col_idx as isize + dc;

            if 0 <= new_row_idx && new_row_idx < nrows && 0 <= new_col_idx && new_col_idx < ncols {
                Some((new_row_idx as usize, new_col_idx as usize))
            } else {
                None
            }
        })
        .skip(1)
    }

    fn build_neighbor_indices(
        floor_mask: &FixedBitSet,
        nrows: usize,
        ncols: usize,
    ) -> Vec<ArrayVec<[usize; 8]>> {
        let mut neighbor_indices = Vec::with_capacity(floor_mask.count_ones(..));

        for chair_idx in floor_mask.ones() {
            let (row_idx, col_idx) = Self::conv_1d_to_2d(ncols, chair_idx);

            let mut idxs = ArrayVec::new();
            for (dr, dc) in DIRECTIONS.iter().copied() {
                for (i, j) in Self::indices_in_direction(nrows, ncols, row_idx, col_idx, dr, dc) {
                    let idx = Self::conv_2d_to_1d(ncols, i, j);
                    if floor_mask.contains(idx) {
                        idxs.push(idx);
                        break;
                    }
                }
            }

            neighbor_indices.push(idxs);
        }

        neighbor_indices
    }

    fn from_str(input: &str) -> Self {
        let mut nrows: usize = 0;
        let mut ncols: usize = 0;
        let elem_iter = input.lines().flat_map(|line| {
            nrows += 1;
            ncols = line.len();
            line.chars().map(|c| match c {
                'L' => true,
                '.' => false,
                _ => panic!("unexpected char: {}", c),
            })
        });

        let chair_idxs = elem_iter.enumerate().filter_map(
            |(idx, is_chair)| {
                if is_chair {
                    Some(idx)
                } else {
                    None
                }
            },
        );

        let floor_mask = FixedBitSet::from_iter(chair_idxs);
        let occupied = FixedBitSet::with_capacity(floor_mask.len());
        let scratch = FixedBitSet::with_capacity(floor_mask.len());
        let neighbor_indices = Self::build_neighbor_indices(&floor_mask, nrows, ncols);

        Self {
            nrows,
            ncols,
            occupied,
            floor_mask,
            scratch,
            neighbor_indices,
        }
    }

    fn hash(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.occupied.hash(&mut hasher);
        hasher.finish()
    }

    fn count_occupied(&self) -> usize {
        self.occupied.count_ones(..)
    }

    fn step(&mut self) {
        self.scratch.clear();

        for (idx, chair_idx) in self.floor_mask.ones().enumerate() {
            let neighbors = &self.neighbor_indices[idx];
            let num_neighbors: usize = neighbors
                .iter()
                .map(|&neighbor_idx| self.occupied.contains(neighbor_idx) as usize)
                .sum();
            let is_occupied = self.occupied.contains(chair_idx);
            let is_now_occupied =
                is_occupied && num_neighbors < 5 || !is_occupied && num_neighbors == 0;

            self.scratch.set(chair_idx, is_now_occupied);
        }

        mem::swap(&mut self.occupied, &mut self.scratch);
    }
}

impl fmt::Display for Layout2 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut str_buf = String::with_capacity(self.ncols + 1);
        for row_idx in 0..self.nrows {
            str_buf.clear();
            for col_idx in 0..self.ncols {
                let idx = Self::conv_2d_to_1d(self.ncols, row_idx, col_idx);
                if !self.floor_mask.contains(idx) {
                    str_buf.push('.');
                } else if self.occupied.contains(idx) {
                    str_buf.push('#');
                } else {
                    str_buf.push('L');
                }
            }
            str_buf.push('\n');
            f.write_str(&str_buf)?;
        }
        Ok(())
    }
}

fn part1(input: &str) {
    let mut layout = Layout::from_str(input);
    let mut iter = 0;
    let mut hash = layout.hash();

    loop {
        layout.step();
        iter += 1;

        let next_hash = layout.hash();
        if next_hash == hash {
            break;
        }

        hash = next_hash;
    }

    dbg!(iter);
    dbg!(layout.count_occupied());
}

fn part2(input: &str) {
    let mut layout = Layout2::from_str(input);
    let mut iter = 0;
    let mut hash = layout.hash();

    loop {
        layout.step();
        iter += 1;

        let next_hash = layout.hash();
        if next_hash == hash {
            break;
        }

        hash = next_hash;
    }

    dbg!(iter);
    dbg!(layout.count_occupied());
}

pub fn run(args: &[&str]) -> Result<()> {
    let input = fs::read_to_string(args[0]).context("Failed to read file")?;

    time!(part1(&input));

    time!(part2(&input));

    Ok(())
}

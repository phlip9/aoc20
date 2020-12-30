use anyhow::{anyhow, Context, Result};
use ndarray::{azip, s, Array, Array2, Array3, Array4, Slice};
use std::{fs, iter::FromIterator};

const BORDER_SIZE: usize = 1;
const MAX_ITERS: usize = 6;

fn parse_input(input: &str) -> Array2<u8> {
    let mut x_len: usize = 0;
    let mut y_len: usize = 0;

    let z_0_iter = input.lines().flat_map(|line| {
        y_len += 1;
        x_len = line.len();
        line.chars().map(|c| match c {
            '#' => 1_u8,
            '.' => 0_u8,
            _ => panic!("unexpected character: '{}'", c),
        })
    });

    let z_0 = Array::from_iter(z_0_iter);
    z_0.into_shape((x_len, y_len)).unwrap()
}

#[derive(Debug)]
struct Cubes {
    active: Array3<u8>,
    scratch: Array3<u8>,
}

impl Cubes {
    fn new(z0: &Array2<u8>) -> Self {
        let (x_len, y_len) = z0.dim();

        let x_len = BORDER_SIZE + MAX_ITERS + x_len + MAX_ITERS + BORDER_SIZE;
        let y_len = BORDER_SIZE + MAX_ITERS + y_len + MAX_ITERS + BORDER_SIZE;
        let z_len = BORDER_SIZE + MAX_ITERS + 1 + MAX_ITERS + BORDER_SIZE;

        let mut active = Array3::zeros((z_len, x_len, y_len));
        let scratch = Array3::zeros((z_len - 2, x_len - 2, y_len - 2));

        const I: isize = BORDER_SIZE as isize + MAX_ITERS as isize;
        active.slice_mut(s![z_len / 2, I..-I, I..-I]).assign(z0);

        Self { active, scratch }
    }

    fn num_active(&self) -> u16 {
        let active = self.active.as_slice_memory_order().unwrap();
        active.iter().map(|&cube| cube as u16).sum()
    }

    fn step(&mut self) {
        let mut neigh = self.scratch.view_mut();
        neigh.fill(0);

        for dz in -1..=1 {
            let z_end = if dz == 1 { None } else { Some(dz - 1) };
            let z_slice = Slice::new(dz + 1, z_end, 1);

            for dx in -1..=1 {
                let x_end = if dx == 1 { None } else { Some(dx - 1) };
                let x_slice = Slice::new(dx + 1, x_end, 1);

                neigh += &self.active.slice(s![z_slice, x_slice, 0..-2]);
                neigh += &self.active.slice(s![z_slice, x_slice, 1..-1]);
                neigh += &self.active.slice(s![z_slice, x_slice, 2..]);
            }
        }

        neigh -= &self.active.slice(s![1..-1, 1..-1, 1..-1]);

        let mut active = self.active.slice_mut(s![1..-1, 1..-1, 1..-1]);
        active.zip_mut_with(&neigh, |a, &n| {
            *a = ((*a == 1 && (n == 2 || n == 3)) || (*a == 0 && n == 3)) as u8
        });
    }
}

#[derive(Debug)]
struct Cubes2 {
    active: Array4<u8>,
    scratch: Array4<u8>,
}

impl Cubes2 {
    fn new(w0z0: &Array2<u8>) -> Self {
        let (x_len, y_len) = w0z0.dim();

        let x_len = BORDER_SIZE + MAX_ITERS + x_len + MAX_ITERS + BORDER_SIZE;
        let y_len = BORDER_SIZE + MAX_ITERS + y_len + MAX_ITERS + BORDER_SIZE;
        let z_len = BORDER_SIZE + MAX_ITERS + 1 + MAX_ITERS + BORDER_SIZE;
        let w_len = BORDER_SIZE + MAX_ITERS + 1 + MAX_ITERS + BORDER_SIZE;

        let mut active = Array4::zeros((w_len, z_len, x_len, y_len));
        let scratch = Array4::zeros((w_len - 2, z_len - 2, x_len - 2, y_len - 2));

        const I: isize = BORDER_SIZE as isize + MAX_ITERS as isize;
        active
            .slice_mut(s![w_len / 2, z_len / 2, I..-I, I..-I])
            .assign(&w0z0);

        Self { active, scratch }
    }

    fn num_active(&self) -> u16 {
        let active = self.active.as_slice_memory_order().unwrap();
        active.iter().map(|&cube| cube as u16).sum()
    }

    fn step(&mut self) {
        let mut neigh = self.scratch.view_mut();
        neigh.fill(0);

        for dw in -1..=1 {
            let w_end = if dw == 1 { None } else { Some(dw - 1) };
            let w_slice = Slice::new(dw + 1, w_end, 1);

            for dz in -1..=1 {
                let z_end = if dz == 1 { None } else { Some(dz - 1) };
                let z_slice = Slice::new(dz + 1, z_end, 1);

                for dx in -1..=1 {
                    let x_end = if dx == 1 { None } else { Some(dx - 1) };
                    let x_slice = Slice::new(dx + 1, x_end, 1);

                    neigh += &self.active.slice(s![w_slice, z_slice, x_slice, 0..-2]);
                    neigh += &self.active.slice(s![w_slice, z_slice, x_slice, 1..-1]);
                    neigh += &self.active.slice(s![w_slice, z_slice, x_slice, 2..]);
                }
            }
        }

        neigh -= &self.active.slice(s![1..-1, 1..-1, 1..-1, 1..-1]);

        let mut active = self.active.slice_mut(s![1..-1, 1..-1, 1..-1, 1..-1]);
        active.zip_mut_with(&neigh, |a, &n| {
            *a = ((*a == 1 && (n == 2 || n == 3)) || (*a == 0 && n == 3)) as u8
        });
    }
}

pub fn run(args: &[&str]) -> Result<()> {
    let input = fs::read_to_string(args[0]).context("Failed to read file")?;
    let z0 = parse_input(&input);

    // part 1
    time!("cubes 1:", {
        let mut cubes = Cubes::new(&z0);
        for _ in 0..6 {
            cubes.step();
        }
        dbg!(cubes.num_active());
    });

    // part 2
    time!("cubes 2:", {
        let mut cubes = Cubes2::new(&z0);
        for _ in 0..6 {
            cubes.step();
        }
        dbg!(cubes.num_active());
    });

    Ok(())
}

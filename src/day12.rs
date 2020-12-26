use anyhow::{Context, Result};
use num_complex::Complex;
use std::{fmt, fs};

const NORTH: Complex<i16> = Complex::new(0, 1);
const SOUTH: Complex<i16> = Complex::new(0, -1);
const EAST: Complex<i16> = Complex::new(1, 0);
const WEST: Complex<i16> = Complex::new(-1, 0);

fn heading_from_degree(degree: i16, right: bool) -> Complex<i16> {
    let left_heading = match degree {
        90 => Complex::new(0, 1),
        180 => Complex::new(-1, 0),
        270 => Complex::new(0, -1),
        _ => panic!("invalid degree: {}", degree),
    };
    if right {
        // 3 lefts make a right : )
        left_heading.powu(3)
    } else {
        left_heading
    }
}

#[derive(Copy, Clone, Debug)]
enum Action {
    Forward(i16),
    Translate(Complex<i16>),
    Rotate(Complex<i16>),
}

impl Action {
    fn from_str(input: &str) -> Self {
        use Action::*;
        let (action, value) = input.split_at(1);
        let value = value.parse::<i16>().unwrap();
        match action {
            "N" => Translate(NORTH * value),
            "S" => Translate(SOUTH * value),
            "E" => Translate(EAST * value),
            "W" => Translate(WEST * value),
            "F" => Forward(value),
            "L" => Rotate(heading_from_degree(value, false)),
            "R" => Rotate(heading_from_degree(value, true)),
            _ => panic!("invalid action: {}", action),
        }
    }
}

#[derive(Debug)]
struct Ship {
    position: Complex<i16>,
    heading: Complex<i16>,
}

impl Ship {
    fn new() -> Self {
        Self {
            position: Complex::new(0, 0),
            heading: EAST,
        }
    }

    fn apply_action(mut self, action: Action) -> Self {
        use Action::*;
        match action {
            Forward(distance) => self.position += self.heading * distance,
            Translate(translation) => self.position += translation,
            Rotate(rotation) => self.heading *= rotation,
        }
        self
    }

    fn manhattan_distance(&self) -> i16 {
        self.position.l1_norm()
    }
}

#[derive(Debug)]
struct Ship2 {
    position: Complex<i16>,
    waypoint: Complex<i16>,
}

impl Ship2 {
    fn new() -> Self {
        Self {
            position: Complex::new(0, 0),
            waypoint: 10 * EAST + 1 * NORTH,
        }
    }

    fn apply_action(mut self, action: Action) -> Self {
        use Action::*;
        match action {
            Forward(distance) => self.position += self.waypoint * distance,
            Translate(translation) => self.waypoint += translation,
            Rotate(rotation) => self.waypoint *= rotation,
        }
        self
    }

    fn manhattan_distance(&self) -> i16 {
        self.position.l1_norm()
    }
}

pub fn run(args: &[&str]) -> Result<()> {
    let input = fs::read_to_string(args[0]).context("Failed to read file")?;
    let actions = input.lines().map(Action::from_str).collect::<Vec<_>>();

    // Part 1
    let ship = actions
        .iter()
        .copied()
        .fold(Ship::new(), Ship::apply_action);
    dbg!(&ship);
    dbg!(ship.manhattan_distance());

    // Part 2
    let ship = actions.into_iter().fold(Ship2::new(), Ship2::apply_action);
    dbg!(&ship);
    dbg!(ship.manhattan_distance());

    Ok(())
}

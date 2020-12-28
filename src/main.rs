#![allow(clippy::cast_lossless)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::enum_glob_use)]
#![allow(clippy::shadow_unrelated)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::similar_names)]

use anyhow::{anyhow, Result};
use std::{env, time::Instant};

pub struct Timer {
    file: &'static str,
    line: u32,
    label: &'static str,
    start: Instant,
}

impl Timer {
    pub fn new(file: &'static str, line: u32, label: &'static str) -> Self {
        Self {
            file,
            line,
            label,
            start: Instant::now(),
        }
    }
}

impl Drop for Timer {
    fn drop(&mut self) {
        eprintln!(
            "[{}:{}] {}: time elapsed {:?}",
            self.file,
            self.line,
            self.label,
            self.start.elapsed(),
        );
    }
}

macro_rules! time {
    ($label:expr, $b:block) => {{
        let _timer = $crate::Timer::new(::std::file!(), ::std::line!(), $label);
        $b
    }};
    ($label:expr, $e:expr) => {{
        time!($label, { $e })
    }};
    ($b:block) => {{
        time!("block", $b)
    }};
    ($e:expr) => {{
        time!(::std::stringify!($e), { $e })
    }};
}

mod day1;
mod day10;
mod day11;
mod day12;
mod day13;
mod day14;
mod day2;
mod day3;
mod day4;
mod day5;
mod day6;
mod day7;
mod day8;
mod day9;
mod util;

fn main() -> Result<()> {
    let args = env::args().into_iter().skip(1).collect::<Vec<_>>();
    println!("{:?}", args);

    let (command, rest_slice) = args.split_first().expect("no command");
    let rest_vec = rest_slice.iter().map(String::as_str).collect::<Vec<_>>();
    let rest = rest_vec.as_slice();

    time!("command", {
        match command.as_str() {
            "day1" => day1::run(rest),
            "day2" => day2::run(rest),
            "day3" => day3::run(rest),
            "day4" => day4::run(rest),
            "day5" => day5::run(rest),
            "day6" => day6::run(rest),
            "day7" => day7::run(rest),
            "day8" => day8::run(rest),
            "day9" => day9::run(rest),
            "day10" => day10::run(rest),
            "day11" => day11::run(rest),
            "day12" => day12::run(rest),
            "day13" => day13::run(rest),
            "day14" => day14::run(rest),
            _ => Err(anyhow!("unrecognized command: '{}'", command)),
        }
    })
}

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
mod day2;
mod day3;
mod day4;
mod day5;
mod util;

fn main() -> Result<()> {
    let args = env::args().into_iter().skip(1).collect::<Vec<_>>();
    println!("{:?}", args);

    let (command, rest) = args.split_first().expect("no command");
    let rest = rest.into_iter().map(String::as_str).collect::<Vec<_>>();

    time!("command", {
        match command.as_str() {
            "day1" => day1::run(rest.as_slice()),
            "day2" => day2::run(rest.as_slice()),
            "day3" => day3::run(rest.as_slice()),
            "day4" => day4::run(rest.as_slice()),
            "day5" => day5::run(rest.as_slice()),
            _ => Err(anyhow!("unrecognized command: '{}'", command)),
        }
    })
}

use crate::util::{read_file_bytes, split_bytes_lines};
use anyhow::Result;
use std::str;
use tinyset::SetU32;

const YEAR: u32 = 2020;
const SIZE: usize = 200;

pub fn run(args: &[&str]) -> Result<()> {
    let file_bytes = read_file_bytes(args[0])?;
    let mut inputs = SetU32::with_capacity_and_max(SIZE, YEAR);

    let nums = split_bytes_lines(&file_bytes).map(|piece| {
        let s = str::from_utf8(piece).expect("invalid utf8");
        let num = s.parse::<u32>().expect("invalid number");
        num
    });
    for num in nums {
        inputs.insert(num);
    }

    let (a, b) = time!(two_sum(&inputs, YEAR).unwrap());
    println!("two_sum: a: {}, b: {}, a * b: {}", a, b, a * b);

    let (a, b, c) = time!(three_sum(&inputs, YEAR).unwrap());
    println!(
        "three_sum: a: {}, b: {}, c: {}, a * b * c: {}",
        a,
        b,
        c,
        a * b * c
    );

    Ok(())
}

fn three_sum(inputs: &SetU32, sum: u32) -> Option<(u32, u32, u32)> {
    for a in inputs.iter() {
        if let Some((b, c)) = two_sum(inputs, sum - a) {
            return Some((a, b, c));
        }
    }

    None
}

fn two_sum(inputs: &SetU32, sum: u32) -> Option<(u32, u32)> {
    for input in inputs.iter() {
        if input > sum {
            continue;
        }
        let other = sum - input;
        if inputs.contains(other) {
            return Some((input, other));
        }
    }
    None
}

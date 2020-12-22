use anyhow::{Context, Result};
use ascii::AsciiChar;
use std::{fmt, fs};

const A_LOWER_ASCII: u8 = AsciiChar::a.as_byte();

const RESPONSE_WIDTH: usize = 26;
const RESPONSE_MASK: u32 = (1 << RESPONSE_WIDTH) - 1;

#[derive(Copy, Clone)]
struct ResponseSet(u32);

impl ResponseSet {
    fn from_bytes(bytes: &[u8]) -> Self {
        let mut bits: u32 = 0;
        for byte in bytes {
            let idx = byte - A_LOWER_ASCII;
            bits |= 1 << idx;
        }
        ResponseSet(bits)
    }

    const fn none() -> Self {
        Self(0)
    }

    const fn all() -> Self {
        Self(RESPONSE_MASK)
    }

    fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }

    fn intersect(self, other: Self) -> Self {
        Self(self.0 & other.0)
    }

    fn count_yes(self) -> u32 {
        self.0.count_ones()
    }
}

impl fmt::Debug for ResponseSet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("ResponseSet")
            .field(&format!("{:026b}", self.0))
            .finish()
    }
}

pub fn run(args: &[&str]) -> Result<()> {
    let input = fs::read_to_string(args[0]).context("Failed to read file")?;

    let groups = input.split("\n\n").map(|group_str| {
        group_str
            .split('\n')
            .filter(|line| !line.is_empty())
            .map(|line| ResponseSet::from_bytes(line.as_bytes()))
    });

    let mut union_yes_counts: u32 = 0;
    let mut intersect_yes_counts: u32 = 0;

    for group in groups {
        let mut union_agg = ResponseSet::none();
        let mut intersect_agg = ResponseSet::all();

        for response in group {
            union_agg = union_agg.union(response);
            intersect_agg = intersect_agg.intersect(response);
        }

        union_yes_counts += union_agg.count_yes();
        intersect_yes_counts += intersect_agg.count_yes();
    }

    dbg!(union_yes_counts);
    dbg!(intersect_yes_counts);

    Ok(())
}

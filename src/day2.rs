use crate::util::read_file_bytes;
use anyhow::{Context, Result};
use regex::RegexBuilder;
use std::{iter::Iterator, str};

struct PasswordEntry<'a> {
    min_reps: u8,
    max_reps: u8,
    letter: &'a str,
    password: &'a str,
}

impl<'a> PasswordEntry<'a> {
    fn is_valid_v1(&self) -> bool {
        let times = self.password.matches(self.letter).count();
        let min_reps = self.min_reps as usize;
        let max_reps = self.max_reps as usize;
        min_reps <= times && times <= max_reps
    }

    fn is_valid_v2(&self) -> bool {
        let i1 = (self.min_reps - 1) as usize;
        let i2 = (self.max_reps - 1) as usize;
        let c1 = &self.password[i1..i1 + 1];
        let c2 = &self.password[i2..i2 + 1];
        let l = self.letter;

        (c1 == l) ^ (c2 == l)
    }
}

pub fn run(args: &[&str]) -> Result<()> {
    let file_bytes = read_file_bytes(&args[0])?;
    let file_str = str::from_utf8(&file_bytes).context("File not valid utf8")?;

    let re = RegexBuilder::new(r"^([0-9]+)-([0-9]+) ([a-z]): ([a-z]+)$")
        .multi_line(true)
        .unicode(false)
        .build()
        .context("Failed to build regex")?;

    let entries = re.captures_iter(&file_str).map(|caps| {
        let min_reps = caps.get(1).unwrap().as_str().parse::<u8>().unwrap();
        let max_reps = caps.get(2).unwrap().as_str().parse::<u8>().unwrap();
        let letter = caps.get(3).unwrap().as_str();
        let password = caps.get(4).unwrap().as_str();

        PasswordEntry {
            min_reps,
            max_reps,
            letter,
            password,
        }
    });

    let mut num_valid_v1 = 0;
    let mut num_valid_v2 = 0;

    for entry in entries {
        if entry.is_valid_v1() {
            num_valid_v1 += 1;
        }
        if entry.is_valid_v2() {
            num_valid_v2 += 1;
        }
    }

    dbg!(num_valid_v1);
    dbg!(num_valid_v2);

    Ok(())
}

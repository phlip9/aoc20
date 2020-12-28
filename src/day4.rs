#![allow(clippy::enum_glob_use)]

use anyhow::{anyhow, Context, Result};
use std::{fs, iter::Iterator, str};

#[derive(Debug, Default)]
struct PassportRaw<'a> {
    byr: Option<&'a str>,
    iyr: Option<&'a str>,
    eyr: Option<&'a str>,
    hgt: Option<&'a str>,
    hcl: Option<&'a str>,
    ecl: Option<&'a str>,
    pid: Option<&'a str>,
    cid: Option<&'a str>,
}

impl<'a> PassportRaw<'a> {
    fn try_from_str(s: &'a str) -> Result<Self> {
        let mut passport = PassportRaw::default();

        let fields = s.split_ascii_whitespace();
        for field in fields {
            let mut splits = field.split(':');

            let (key, value) = match (splits.next(), splits.next(), splits.next()) {
                (Some(key), Some(value), None) => (key, value),
                _ => return Err(anyhow!("Invalid field: {}", s)),
            };

            match key {
                "byr" => passport.byr = Some(value),
                "iyr" => passport.iyr = Some(value),
                "eyr" => passport.eyr = Some(value),
                "hgt" => passport.hgt = Some(value),
                "hcl" => passport.hcl = Some(value),
                "ecl" => passport.ecl = Some(value),
                "pid" => passport.pid = Some(value),
                "cid" => passport.cid = Some(value),
                _ => return Err(anyhow!("Invalid field name: {}", key)),
            }
        }

        Ok(passport)
    }
}

struct PassportV1<'a> {
    byr: &'a str,
    iyr: &'a str,
    eyr: &'a str,
    hgt: &'a str,
    hcl: &'a str,
    ecl: &'a str,
    pid: &'a str,
}

impl<'a> PassportV1<'a> {
    fn try_from_raw(raw: &PassportRaw<'a>) -> Option<Self> {
        Some(PassportV1 {
            byr: raw.byr?,
            iyr: raw.iyr?,
            eyr: raw.eyr?,
            hgt: raw.hgt?,
            hcl: raw.hcl?,
            ecl: raw.ecl?,
            pid: raw.pid?,
        })
    }
}

fn parse_num_range(s: &str, min: u32, max: u32) -> Result<u32> {
    let num = s.parse::<u32>()?;
    if min <= num && num <= max {
        Ok(num)
    } else {
        Err(anyhow!("value out of range: {}", num))
    }
}

enum Height {
    In(u32),
    Cm(u32),
}

impl Height {
    fn try_from_str(s: &str) -> Result<Self> {
        use Height::*;
        if let Some(s) = s.strip_suffix("in") {
            Ok(In(parse_num_range(s, 59, 76)?))
        } else if let Some(s) = s.strip_suffix("cm") {
            Ok(Cm(parse_num_range(s, 150, 193)?))
        } else {
            Err(anyhow!("invalid height units"))
        }
    }
}

fn parse_hair_color(s: &str) -> Option<&str> {
    let rest = s.strip_prefix('#')?;
    if rest.len() != 6 {
        return None;
    }
    for c in rest.chars() {
        match c {
            '0'..='9' | 'a'..='f' => (),
            _ => return None,
        }
    }
    Some(s)
}

fn parse_eye_color(s: &str) -> Option<&str> {
    match s {
        "amb" | "blu" | "brn" | "gry" | "grn" | "hzl" | "oth" => Some(s),
        _ => None,
    }
}

fn parse_passport_id(s: &str) -> Option<&str> {
    if s.len() != 9 {
        return None;
    }
    for c in s.chars() {
        match c {
            '0'..='9' => (),
            _ => return None,
        }
    }
    Some(s)
}

struct PassportV2<'a> {
    _byr: u32,
    _iyr: u32,
    _eyr: u32,
    _hgt: Height,
    _hcl: &'a str,
    _ecl: &'a str,
    _pid: &'a str,
}

impl<'a> PassportV2<'a> {
    fn try_from_v1(raw: &PassportV1<'a>) -> Result<Self> {
        let byr = parse_num_range(raw.byr, 1920, 2002).context("Invalid birth year")?;
        let iyr = parse_num_range(raw.iyr, 2010, 2020).context("Invalid issue year")?;
        let eyr = parse_num_range(raw.eyr, 2020, 2030).context("Invalid expiration")?;
        let hgt = Height::try_from_str(raw.hgt).context("Invalid height")?;
        let hcl = parse_hair_color(raw.hcl).ok_or_else(|| anyhow!("Invalid hair color"))?;
        let ecl = parse_eye_color(raw.ecl).ok_or_else(|| anyhow!("Invalid eye color"))?;
        let pid = parse_passport_id(raw.pid).ok_or_else(|| anyhow!("Invalid passport id"))?;

        Ok(Self {
            _byr: byr,
            _iyr: iyr,
            _eyr: eyr,
            _hgt: hgt,
            _hcl: hcl,
            _ecl: ecl,
            _pid: pid,
        })
    }
}

pub fn run(args: &[&str]) -> Result<()> {
    let input = fs::read_to_string(args[0]).context("Failed to read file")?;

    let mut valid_count_v1: u32 = 0;
    let mut valid_count_v2: u32 = 0;
    for passport_str in input.split("\n\n") {
        let passport_raw =
            PassportRaw::try_from_str(passport_str).context("Failed to parse passport")?;
        let passport_v1 = PassportV1::try_from_raw(&passport_raw);
        if let Some(passport_v1) = passport_v1 {
            valid_count_v1 += 1;
            valid_count_v2 += PassportV2::try_from_v1(&passport_v1).is_ok() as u32;
        }
    }

    dbg!(valid_count_v1);
    dbg!(valid_count_v2);

    Ok(())
}

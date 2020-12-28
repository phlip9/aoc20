use anyhow::{Context, Result};
use nom::{
    branch::alt,
    bytes::complete::{tag, take},
    character::complete::{digit1, multispace0, newline},
    combinator::{all_consuming, eof, map, map_opt},
    multi::separated_list0,
    sequence::{preceded, terminated, tuple},
    IResult,
};
use std::{collections::HashMap, fmt, fs, str::FromStr};

const BITS: u8 = 36;
const VALUE_MASK: u64 = (1 << BITS) - 1;

#[derive(Copy, Clone)]
enum Action {
    SetMask { one_mask: u64, zero_mask: u64 },
    SetMem { addr: u64, value: u64 },
}

impl Action {
    fn parse(s: &str) -> IResult<&str, Action> {
        alt((parse_set_mask, parse_set_mem))(s)
    }
}

impl fmt::Display for Action {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use Action::*;
        match self {
            SetMask {
                one_mask,
                zero_mask,
            } => {
                let mut mask_string = String::with_capacity(BITS as usize);
                for i in (0..BITS).rev() {
                    let mask = 1 << i;
                    if one_mask & mask != 0 {
                        mask_string.push('1');
                    } else if zero_mask & mask != 0 {
                        mask_string.push('0');
                    } else {
                        mask_string.push('X');
                    }
                }
                write!(f, "mask = {}", mask_string)
            }
            SetMem { addr, value } => write!(f, "mem[{}] = {}", addr, value),
        }
    }
}

impl fmt::Debug for Action {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use Action::*;
        match self {
            SetMask {
                one_mask,
                zero_mask,
            } => f
                .debug_struct("Action::SetMask")
                .field("one_mask", &format!("{:036b}", one_mask))
                .field("zero_mask", &format!("{:036b}", zero_mask))
                .finish(),
            SetMem { addr, value } => f
                .debug_struct("Action::SetMem")
                .field("addr", &addr)
                .field("value", &value)
                .finish(),
        }
    }
}

fn parse_mask_bits(s: &str) -> Option<Action> {
    let mut one_mask: u64 = 0;
    let mut zero_mask: u64 = 0;

    for (idx, c) in s.chars().enumerate() {
        let idx = idx as u8;
        let bit = BITS - idx - 1;
        match c {
            '0' => zero_mask |= 1 << bit,
            '1' => one_mask |= 1 << bit,
            'X' => (),
            _ => return None,
        }
    }

    Some(Action::SetMask {
        one_mask,
        zero_mask,
    })
}

// mask = <set-mask>
fn parse_set_mask(s: &str) -> IResult<&str, Action> {
    map_opt(preceded(tag("mask = "), take(BITS)), parse_mask_bits)(s)
}

// mem[<addr>] = <value>
fn parse_set_mem(s: &str) -> IResult<&str, Action> {
    map(
        tuple((
            map_opt(preceded(tag("mem["), digit1), |s| u64::from_str(s).ok()),
            map_opt(preceded(tag("] = "), digit1), |s| u64::from_str(s).ok()),
        )),
        |(addr, value)| Action::SetMem { addr, value },
    )(s)
}

fn parse_all_actions(s: &str) -> Vec<Action> {
    let end = tuple((multispace0, eof));
    let (_rest, actions) =
        all_consuming(terminated(separated_list0(newline, Action::parse), end))(s).unwrap();
    actions
}

struct Memory {
    mem: HashMap<u64, u64>,
    one_mask: u64,
    zero_mask: u64,
    floating_mask: u64,
}

impl Memory {
    fn new() -> Self {
        Self {
            mem: HashMap::new(),
            one_mask: 0,
            zero_mask: 0,
            floating_mask: 0,
        }
    }

    fn apply_action_v1(mut self, action: Action) -> Self {
        use Action::*;
        match action {
            SetMask {
                one_mask,
                zero_mask,
            } => {
                self.one_mask = one_mask;
                self.zero_mask = zero_mask;
            }
            SetMem { addr, value } => {
                let value = (value | self.one_mask) & !self.zero_mask;
                let value = value & VALUE_MASK;
                self.mem.insert(addr, value);
            }
        }
        self
    }

    #[cfg(target_arch = "x86_64")]
    #[target_feature(enable = "bmi2")]
    unsafe fn apply_action_v2(mut self, action: Action) -> Self {
        use Action::*;
        match action {
            SetMask {
                one_mask,
                zero_mask,
            } => {
                self.one_mask = one_mask;
                self.zero_mask = zero_mask;
                self.floating_mask = !(one_mask | zero_mask) & VALUE_MASK;
            }
            SetMem { addr, value } => {
                let addr = addr | self.one_mask;
                // remove the floating bits from the address
                let addr = addr & !self.floating_mask;

                for floating_mask in mask_permutations(self.floating_mask) {
                    // only add this specific permutation's floating bits back
                    // into the address
                    let addr = addr | floating_mask;
                    self.mem.insert(addr, value);
                }
            }
        }
        self
    }

    fn sum(&self) -> u64 {
        self.mem.values().sum()
    }
}

impl fmt::Debug for Memory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Memory")
            .field("mem", &self.mem)
            .field("one_mask ", &format!("{:036b}", self.one_mask))
            .field("zero_mask", &format!("{:036b}", self.zero_mask))
            .field("floating_mask", &format!("{:036b}", self.floating_mask))
            .finish()
    }
}

// Get all the different permutations of the bits in a mask. For example,
//
// ### Example:
//
// mask_permutations(1101) = [ 0000 0001 0100 0101 1000 1001 1100 1101 ]
#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "bmi2")]
unsafe fn mask_permutations(mask: u64) -> impl Iterator<Item = u64> {
    // _pdep_u64 deposits contiguous low bits from unsigned 64-bit integer a to
    // dst at the corresponding bit locations specified by mask; all other bits
    // in dst are set to zero.
    use core::arch::x86_64::_pdep_u64;

    let num_permutations = 1 << mask.count_ones();

    // Generate each permutation of bits as a contiguous chunk (the index), then
    // spread the contiguous bits into the floating mask bits (sparse bits).
    //
    // For example, a sinlge index spread into the mask:
    //
    // mask 1001100101
    // idx  0000011001
    //           |||||
    //           ////|
    //          ///| |
    //         /// | |
    //        ///  | |
    //       / ||  | |
    //      |  ||  | |
    // out  1001000001
    (0..num_permutations)
        .into_iter()
        .map(move |index| _pdep_u64(index, mask))
}

fn part1(actions: &[Action]) {
    let memory = actions
        .iter()
        .copied()
        .fold(Memory::new(), Memory::apply_action_v1);

    dbg!(memory.sum());
    dbg!(memory.mem.len());
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "bmi2")]
unsafe fn part2(actions: &[Action]) {
    let memory = actions
        .iter()
        .copied()
        .fold(Memory::new(), |memory, action| {
            memory.apply_action_v2(action)
        });

    dbg!(memory.sum());
    dbg!(memory.mem.len());
}

pub fn run(args: &[&str]) -> Result<()> {
    let input = fs::read_to_string(args[0]).context("Failed to read file")?;

    let actions = time!(parse_all_actions(&input));

    time!(part1(&actions));

    #[cfg(target_arch = "x86_64")]
    #[target_feature(enable = "bmi2")]
    unsafe {
        time!(part2(&actions))
    };

    Ok(())
}

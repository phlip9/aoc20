use anyhow::{Context, Result};
use std::{
    collections::{HashSet, VecDeque},
    fs,
    iter::FromIterator,
};

const PREAMBLE_LEN: usize = 25;

fn has_two_sum(inputs: &HashSet<u64>, sum: u64) -> bool {
    for x in inputs.iter().copied() {
        if x >= sum {
            continue;
        }
        let y = sum - x;
        if x != y && inputs.contains(&y) {
            return true;
        }
    }
    false
}

fn find_invalid(nums: &[u64]) -> Option<(usize, u64)> {
    let (preamble_slice, nums) = nums.split_at(PREAMBLE_LEN);

    let mut preamble = preamble_slice.iter().copied().collect::<VecDeque<_>>();
    let mut preamble_set = HashSet::from_iter(preamble_slice.iter().copied());

    for (idx, num) in nums.iter().copied().enumerate() {
        if !has_two_sum(&preamble_set, num) {
            // couldn't find pair sum
            return Some((idx, num));
        }

        // remove oldest preamble entry, add new num

        let front = preamble.pop_front().expect("preamble cannot be empty");
        preamble_set.remove(&front);

        preamble.push_back(num);
        preamble_set.insert(num);
    }

    None
}

fn find_contiguous_ksum(nums: &[u64], sum: u64) -> &[u64] {
    let mut window_range = 0..0;
    let mut window_sum = 0;

    loop {
        // expand window until we pass sum
        while window_sum < sum {
            window_sum += nums[window_range.end];
            window_range.end += 1;
        }

        // contract window until we go back under
        while window_sum > sum {
            window_sum -= nums[window_range.start];
            window_range.start += 1;
        }

        // found a ksum window
        if window_sum == sum {
            return &nums[window_range];
        }
    }
}

pub fn run(args: &[&str]) -> Result<()> {
    let input = fs::read_to_string(args[0]).context("Failed to read file")?;
    let nums = input
        .lines()
        .map(|line| line.parse::<u64>().expect("failed to parse num"))
        .collect::<Vec<_>>();

    // Part 1
    let (invalid_idx, invalid_num) = dbg!(find_invalid(&nums).expect("no invalid number"));

    // Part 2
    let ksum = find_contiguous_ksum(&nums[..invalid_idx], invalid_num);
    let min = ksum.iter().min().unwrap();
    let max = ksum.iter().max().unwrap();

    dbg!(ksum, min, max, min + max);

    Ok(())
}

use anyhow::{Context, Result};
use std::{collections::HashMap, fs, num::NonZeroU32};

#[derive(Debug)]
struct Game {
    round: u32,
    prev_num_spoken: u32,
    prev_round_spoken: HashMap<u32, (u32, Option<NonZeroU32>)>,
}

impl Game {
    fn new(starting_numbers: &[u32]) -> Self {
        let prev_round_spoken = starting_numbers
            .iter()
            .enumerate()
            .map(|(round, &num)| (num, ((round + 1) as u32, None)))
            .collect::<HashMap<_, _>>();

        let prev_num_spoken = *starting_numbers.last().unwrap();

        Self {
            round: starting_numbers.len() as u32,
            prev_num_spoken,
            prev_round_spoken,
        }
    }

    fn speak(&mut self, num: u32) -> u32 {
        self.prev_num_spoken = num;

        let maybe_prev_round_spoken = self
            .prev_round_spoken
            .get(&num)
            .map(|(prev_round_spoken, _)| *prev_round_spoken)
            .and_then(NonZeroU32::new);

        self.prev_round_spoken
            .insert(num, (self.round, maybe_prev_round_spoken));

        num
    }

    fn step(&mut self) -> u32 {
        self.round += 1;

        let (prev_round_spoken, prev_prev_round_spoken) =
            self.prev_round_spoken[&self.prev_num_spoken];

        match prev_prev_round_spoken {
            // first time prev number was spoken; say a 0
            None => self.speak(0),
            // we've already seen this number; say the difference b/w its
            // prev_round_spoken and its prev_prev_round_spoken
            Some(prev_prev_round_spoken) => {
                let diff = prev_round_spoken - prev_prev_round_spoken.get();
                self.speak(diff)
            }
        }
    }

    fn step_until_round(&mut self, round: u32) -> u32 {
        loop {
            let num = self.step();
            if self.round == round {
                return num;
            }
        }
    }
}

pub fn run(args: &[&str]) -> Result<()> {
    let input = fs::read_to_string(args[0]).context("Failed to read file")?;

    let line = input.lines().next().unwrap();
    let starting_numbers = line
        .split(',')
        .map(|slice| slice.parse::<u32>().unwrap())
        .collect::<Vec<_>>();

    let mut game = Game::new(&starting_numbers);

    // part 1
    dbg!(game.step_until_round(2020));

    // part 2
    dbg!(game.step_until_round(30_000_000));

    Ok(())
}

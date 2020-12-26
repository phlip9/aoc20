use anyhow::{Context, Result};
use std::{fs, iter};

fn diffs_distribution(adapters: &[u8]) -> [u8; 3] {
    let mut distr = [0u8; 3];
    let diffs = adapters.windows(2).map(|slice| slice[1] - slice[0]);
    for diff in diffs {
        distr[(diff - 1) as usize] += 1;
    }
    distr
}

// N = { adapters }
// E = { (i, j) : i + 3 >= j && N_i + 3 >= N_j }_{ (i, j) in {0..n-1}^2 }
// G = (N, E)
//
// observe: G is a DAG, already topologically sorted
//
// paths_i : number of unique paths to terminal from node/adapter i
// paths_n = 1
// paths_i = sum_{j : (i, j) in E} paths_j
// paths_0 == # unique valid adapter arrangements

fn count_paths(adapters: &[u8]) -> u64 {
    let n = adapters.len();
    let mut paths = vec![0u64; n];
    paths[n - 1] = 1;

    for i in (0..n - 1).rev() {
        let mut paths_i = 0;
        let a_i = adapters[i];

        // for e_ij in E
        for j in i + 1..=i + 3 {
            if let Some(a_j) = adapters.get(j) {
                if a_i + 3 >= *a_j {
                    paths_i += paths[j];
                } else {
                    break;
                }
            }
        }

        paths[i] = paths_i;

        println!("paths[{}] = {}", i, paths_i);
    }

    paths[0]
}

pub fn run(args: &[&str]) -> Result<()> {
    let input = fs::read_to_string(args[0]).context("Failed to read file")?;
    let adapters = input.lines().map(|line| line.parse::<u8>().unwrap());
    let mut adapters = iter::once(0).chain(adapters).collect::<Vec<_>>();
    adapters.sort_unstable();
    adapters.push(adapters.last().unwrap() + 3);

    // Part 1
    let diffs_distr = diffs_distribution(&adapters);
    dbg!(diffs_distr);
    dbg!(diffs_distr[1 - 1] as usize * diffs_distr[3 - 1] as usize);

    // Part 2
    dbg!(count_paths(&adapters));

    Ok(())
}

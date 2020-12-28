use crate::util::{read_file_bytes, split_bytes_lines};
use anyhow::Result;
use std::{fmt, iter::Iterator, str};

const WIDTH: usize = 31;
const OPEN_CHAR: char = '.';
const TREE: u8 = 35;
const TREE_CHAR: char = '#';

struct Horizontal {
    trees: u32,
}

impl Horizontal {
    fn from_line(line: &[u8]) -> Self {
        let mut trees = 0;
        for (idx, byte) in line.iter().enumerate() {
            if *byte == TREE {
                trees |= 1 << idx;
            }
        }
        Self { trees }
    }

    const fn is_tree_inner(&self, x: u8) -> bool {
        let mask = 1 << x;
        self.trees & mask != 0
    }

    const fn is_tree(&self, x: usize) -> bool {
        self.is_tree_inner((x % WIDTH) as u8)
    }
}

impl fmt::Display for Horizontal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut line = String::with_capacity(WIDTH);
        for x in 0..WIDTH as u8 {
            if self.is_tree_inner(x) {
                line.push(TREE_CHAR);
            } else {
                line.push(OPEN_CHAR);
            }
        }
        f.write_str(&line)
    }
}

struct Geology {
    horizontals: Vec<Horizontal>,
}

impl Geology {
    fn from_lines<'a>(lines: impl Iterator<Item = &'a [u8]>) -> Self {
        let horizontals = lines.map(Horizontal::from_line).collect::<Vec<_>>();
        Self { horizontals }
    }

    fn height(&self) -> usize {
        self.horizontals.len()
    }

    fn horizontal(&self, y: usize) -> &Horizontal {
        &self.horizontals[y]
    }

    fn is_tree(&self, x: usize, y: usize) -> bool {
        self.horizontal(y).is_tree(x)
    }

    fn count_trees(&self, dx: usize, dy: usize) -> usize {
        let height = self.height();

        let mut count = 0;
        let mut x = 0;
        let mut y = 0;

        while y < height {
            count += self.is_tree(x, y) as usize;
            x += dx;
            y += dy;
        }

        count
    }
}

impl fmt::Display for Geology {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for horiz in &self.horizontals {
            let line = horiz.to_string();
            writeln!(f, "{}", line)?;
        }
        Ok(())
    }
}

pub fn run(args: &[&str]) -> Result<()> {
    let file_bytes = read_file_bytes(args[0])?;
    let lines = split_bytes_lines(&file_bytes);
    let geology = Geology::from_lines(lines);

    let slopes: [(usize, usize); 5] = [(1, 1), (3, 1), (5, 1), (7, 1), (1, 2)];

    let product: usize = slopes
        .iter()
        .map(|(dx, dy)| {
            let count = geology.count_trees(*dx, *dy);
            println!("dx: {}, dy: {}, count: {}", dx, dy, count);
            count
        })
        .product();

    dbg!(product);

    Ok(())
}

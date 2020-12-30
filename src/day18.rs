use anyhow::{Context, Result};
use std::fs;

#[derive(Ord, PartialOrd, Eq, PartialEq)]
enum Token {
    Num(u64),
    Mul,
    Add,
    LParen,
    RParen,
}

fn tokenize(s: &str) -> Vec<Token> {
    use Token::*;
    s.chars()
        .filter_map(|c| match c {
            '0'..='9' => c.to_digit(10).map(|d| Num(d as u64)),
            '+' => Some(Add),
            '*' => Some(Mul),
            '(' => Some(LParen),
            ')' => Some(RParen),
            _ => None,
        })
        .collect::<Vec<_>>()
}

fn find_matching_lparen(tokens: &[Token]) -> Option<usize> {
    use Token::*;

    let mut depth = 0;
    for (idx, token) in tokens.iter().enumerate().rev() {
        match token {
            RParen => depth += 1,
            LParen if depth == 0 => return Some(idx),
            LParen => depth -= 1,
            _ => (),
        }
    }
    None
}

fn split_lowest_precedence(tokens: &[Token], v2: bool) -> Option<(&[Token], &Token, &[Token])> {
    use Token::*;

    let mut lowest_idx = None;
    let mut idx = tokens.len();

    while idx > 0 {
        idx -= 1;
        match tokens[idx] {
            Add => {
                if v2 {
                    if lowest_idx.is_none() {
                        lowest_idx = Some(idx);
                    }
                } else {
                    lowest_idx = Some(idx);
                    break;
                }
            }
            Mul => {
                lowest_idx = Some(idx);
                break;
            }
            RParen => {
                let lparen = find_matching_lparen(&tokens[..idx]).expect("no matching rparen");
                idx = lparen;
            }
            Num(_) => (),
            LParen => panic!("unexpected lparen"),
        }
    }

    let lowest_idx = lowest_idx?;

    let (left, right) = tokens.split_at(lowest_idx);
    let mid = &right[0];
    let right = &right[1..];
    Some((left, mid, right))
}

fn eval(tokens: &[Token], v2: bool) -> u64 {
    use Token::*;

    if let [Num(n)] = tokens {
        return *n;
    }

    match split_lowest_precedence(tokens, v2) {
        Some((left, mid, right)) => {
            let left = eval(left, v2);
            let right = eval(right, v2);

            match mid {
                Add => left + right,
                Mul => left * right,
                _ => panic!("unexpected mid token"),
            }
        }
        None => match tokens {
            [LParen, inner @ .., RParen] => eval(inner, v2),
            _ => panic!("expected outer parens"),
        },
    }
}

fn eval_str_v1(input: &str) -> u64 {
    let tokens = tokenize(input);
    eval(&tokens, false)
}

fn eval_str_v2(input: &str) -> u64 {
    let tokens = tokenize(input);
    eval(&tokens, true)
}

pub fn run(args: &[&str]) -> Result<()> {
    let input = fs::read_to_string(args[0]).context("Failed to read file")?;

    dbg!(input.lines().map(eval_str_v1).sum::<u64>());
    dbg!(input.lines().map(eval_str_v2).sum::<u64>());

    Ok(())
}

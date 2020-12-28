use anyhow::{Context, Result};
use std::fs;

// find x, y, d in ℤ : a x + b y = d, d = gcd(a, b)
#[allow(clippy::many_single_char_names)]
fn egcd(a: i64, b: i64) -> (i64, i64, i64) {
    let (mut r_p, mut r) = (a, b);
    let (mut s_p, mut s) = (1, 0);
    let (mut t_p, mut t) = (0, 1);

    while r != 0 {
        let q = r_p / r;

        let r_t = r_p - q * r;
        r_p = r;
        r = r_t;

        let s_t = s_p - q * s;
        s_p = s;
        s = s_t;

        let t_t = t_p - q * t;
        t_p = t;
        t = t_t;
    }

    let d = r_p;
    let x = s_p;
    let y = t_p;

    assert_eq!(a * x + b * y, d);

    (x, y, d)
}

// find a⁻¹ in ℤ : a a⁻¹ ≡ 1 mod m
fn modinv(a: i64, m: i64) -> Option<i64> {
    let (inv_a, _, d) = egcd(a, m);
    // a, m must be coprime
    if d == 1 {
        let inv_a = inv_a.rem_euclid(m);
        assert_eq!((a * inv_a).rem_euclid(m), 1);
        Some(inv_a)
    } else {
        // a, m are _not_ coprime, no modular inverse
        None
    }
}

// Chinese Remainder Theorem:
// ==========================
//
// n_1 .. n_k in ℤ
// a_1 .. a_k in ℤ
//
// x ≡ a_1 mod n_1
//   ⋮
// x ≡ a_k mod n_k
//
// N = n_1 * .. * n_k
// N_i = N / n_i
// M_i = modinv(N_i, n_i)
//
// x = sum a_i M_i N_i
#[allow(non_snake_case)]
fn chinese_remainder_theorem(a: &[i64], n: &[i64]) -> Option<i64> {
    let N: i64 = n.iter().product();

    a.iter()
        .zip(n.iter())
        .map(|(a_i, &n_i)| {
            let N_i = N / n_i;
            let M_i = modinv(N_i, n_i)?;
            Some(a_i * M_i * N_i)
        })
        .sum::<Option<i64>>()
        .map(|sum| sum.rem_euclid(N))
}

// find bus with earliest arrival time after `earliest_timestamp`
fn part1(input: &str) {
    let mut lines = input.lines();

    let earliest_timestamp = lines.next().unwrap().parse::<i64>().unwrap();
    dbg!(&earliest_timestamp);

    let bus_arrivals = lines
        .next()
        .unwrap()
        .split(',')
        .filter_map(|maybe_freq| maybe_freq.parse::<i64>().ok());

    let (delay_until_arrival, freq) = bus_arrivals
        .map(|freq| {
            //   f - (t mod f) mod f
            // = (f - t)       mod f
            // = -t            mod f
            let delay_until_arrival = (-earliest_timestamp).rem_euclid(freq);
            (delay_until_arrival, freq)
        })
        .min()
        .unwrap();

    dbg!(delay_until_arrival, freq, delay_until_arrival * freq);
}

// example: 7,13,x,x,59,x,31,19
// 7  13 _  _  59 _  31 19
// 0  1  2  3  4  5  6  7
//
// find x in Z_+ s.t.
//
// x =  0 = ( 7 - 0) mod  7
// x = 12 = (13 - 1) mod 13
// x = 55 = (59 - 4) mod 59
// x = 25 = (31 - 6) mod 31
// x = 12 = (19 - 7) mod 19
//
// 7, 13, 59, 31, 19 are coprime
// ==> find x using the Chinese Remainder Theorem : )
fn part2(input: &str) {
    let line = input.lines().nth(1).unwrap();

    let (a, n): (Vec<i64>, Vec<i64>) = line
        .split(',')
        .enumerate()
        .filter_map(|(i, n_i_str)| {
            n_i_str
                .parse::<i64>()
                .ok()
                .map(|n_i| ((n_i - (i as i64)).rem_euclid(n_i), n_i))
        })
        .unzip();

    let x = chinese_remainder_theorem(&a, &n).unwrap();

    dbg!(x);
}

pub fn run(args: &[&str]) -> Result<()> {
    let input = fs::read_to_string(args[0]).context("Failed to read file")?;

    time!(part1(&input));
    time!(part2(&input));

    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_egcd() {
        let (x, y, d) = egcd(240, 46);
        assert_eq!(x, -9);
        assert_eq!(y, 47);
        assert_eq!(d, 2);
    }

    #[test]
    fn test_modinv() {
        let a_inv = modinv(5, 9).unwrap();
        assert_eq!(a_inv, 2);
    }

    #[test]
    fn test_crt() {
        // The classic example : )
        let a = [2, 3, 2];
        let n = [3, 5, 7];
        assert_eq!(Some(23), chinese_remainder_theorem(&a, &n));
    }
}

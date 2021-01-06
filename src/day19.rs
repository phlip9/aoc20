use anyhow::{Context, Result};
use itertools::Itertools;
use std::{fmt, fs};

#[derive(Clone, Debug, Eq, PartialEq)]
enum Rule {
    Or((u8, u8), (u8, u8)),
    Or2(u8, u8),
    Concat(u8, u8),
    Alias(u8),
    A,
    B,
    // 8: x | x 8
    // (x .. x) ..
    Or8(u8),
    // 11: x y | x 11 y
    // (x .. x y .. y) ..
    Or11(u8, u8),
    Empty,
}

impl Rule {
    fn parse_concat(s: &str) -> Option<(u8, u8)> {
        s.split(' ').collect_tuple().and_then(|(s1, s2)| {
            let i1 = s1.parse::<u8>().ok()?;
            let i2 = s2.parse::<u8>().ok()?;
            Some((i1, i2))
        })
    }

    fn parse(s: &str) -> Self {
        let mut splits = s.split(" | ");

        match (splits.next(), splits.next(), splits.next()) {
            (Some(s), None, None) => {
                if let Some((i1, i2)) = Self::parse_concat(s) {
                    Self::Concat(i1, i2)
                } else if let Ok(i) = s.parse::<u8>() {
                    Self::Alias(i)
                } else if s == "\"a\"" {
                    Self::A
                } else if s == "\"b\"" {
                    Self::B
                } else {
                    panic!("bad base rule: {}", s)
                }
            }
            (Some(s1), Some(s2), None) => {
                // println!("Some(s1), Some(s2) = {}, {}", s1, s2);

                if let (Some(c1), Some(c2)) = (Self::parse_concat(s1), Self::parse_concat(s2)) {
                    Self::Or(c1, c2)
                } else if let (Some(i1), Some(i2)) = (s1.parse::<u8>().ok(), s2.parse::<u8>().ok())
                {
                    Self::Or2(i1, i2)
                } else {
                    panic!("bad or rule: {}", s)
                }
            }
            _ => panic!("bad rule: {}", s),
        }
    }
}

const MAX_RULES: usize = 150;

#[derive(Eq, PartialEq)]
struct Rules {
    rules: Vec<Rule>,
}

impl Rules {
    fn parse(s: &str, v2: bool) -> Self {
        let mut rules = vec![Rule::Empty; MAX_RULES];

        for line in s.lines() {
            let (idx, rule) = line.split(": ").collect_tuple().unwrap();
            let idx = idx.parse::<u8>().unwrap();
            let rule = if v2 && idx == 8 {
                Rule::Or8(42)
            } else if v2 && idx == 11 {
                Rule::Or11(42, 31)
            } else {
                Rule::parse(rule)
            };

            rules[idx as usize] = rule;
        }

        Self { rules }
    }

    fn parse_v1(s: &str) -> Self {
        Self::parse(s, false)
    }
    fn parse_v2(s: &str) -> Self {
        Self::parse(s, true)
    }
}

impl fmt::Debug for Rules {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (idx, rule) in self.rules.iter().enumerate() {
            writeln!(f, "{}: {:?}", idx, rule)?;
        }
        Ok(())
    }
}

const MAX_DEPTH: usize = 5;

fn build_regexes(regexes: &mut Vec<String>, rules: &[Rule], id: u8) {
    if !regexes[id as usize].is_empty() {
        return;
    }

    let rule = &rules[id as usize];

    let regex_string = match rule {
        Rule::A => "a".to_string(),
        Rule::B => "b".to_string(),
        Rule::Alias(id) => {
            build_regexes(regexes, rules, *id);
            let r = &regexes[*id as usize];
            r.to_string()
        }
        Rule::Concat(id1, id2) => {
            build_regexes(regexes, rules, *id1);
            build_regexes(regexes, rules, *id2);

            let r1 = &regexes[*id1 as usize];
            let r2 = &regexes[*id2 as usize];

            format!("{}{}", r1, r2)
        }
        Rule::Or((id11, id12), (id21, id22)) => {
            build_regexes(regexes, rules, *id11);
            build_regexes(regexes, rules, *id12);
            build_regexes(regexes, rules, *id21);
            build_regexes(regexes, rules, *id22);

            let r11 = &regexes[*id11 as usize];
            let r12 = &regexes[*id12 as usize];
            let r21 = &regexes[*id21 as usize];
            let r22 = &regexes[*id22 as usize];

            format!("({}{}|{}{})", r11, r12, r21, r22)
        }
        Rule::Or2(id1, id2) => {
            build_regexes(regexes, rules, *id1);
            build_regexes(regexes, rules, *id2);

            let r1 = &regexes[*id1 as usize];
            let r2 = &regexes[*id2 as usize];

            format!("({}|{})", r1, r2)
        }
        Rule::Or8(id) => {
            build_regexes(regexes, rules, *id);

            let r1 = &regexes[*id as usize];

            format!("({})+", r1)
        }
        Rule::Or11(id1, id2) => {
            build_regexes(regexes, rules, *id1);
            build_regexes(regexes, rules, *id2);

            let r1 = &regexes[*id1 as usize];
            let r2 = &regexes[*id2 as usize];

            // (r1){1}(r2){1} | (r1){2}(r2){2} | ...
            let cases = (1..MAX_DEPTH)
                .map(|i| format!("{}{{{}}}{}{{{}}}", r1, i, r2, i))
                .join("|");

            format!("({})", cases)
        }
        Rule::Empty => panic!("empty rule: id: {}", id),
    };

    regexes[id as usize] = regex_string;
}

fn run_regexes(rules: &[Rule], inputs: &str) {
    let mut regexes = vec![String::new(); MAX_RULES];

    time!(build_regexes(&mut regexes, rules, 0));

    let base_regex = regex::RegexBuilder::new(&format!("^{}$", &regexes[0]))
        .unicode(false)
        .build()
        .unwrap();

    let matching_lines = inputs.lines().filter(|line| base_regex.is_match(line));
    let num_matching = time!(matching_lines.count());

    dbg!(num_matching);
}

pub fn run(args: &[&str]) -> Result<()> {
    let input = fs::read_to_string(args[0]).context("Failed to read file")?;

    let (rules_str, inputs) = input.split("\n\n").collect_tuple().unwrap();

    // part 1
    let rules_v1 = Rules::parse_v1(rules_str);
    time!(run_regexes(&rules_v1.rules, inputs));

    // part 2
    let rules_v2 = Rules::parse_v2(rules_str);
    time!(run_regexes(&rules_v2.rules, inputs));

    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_parse_rule() {
        use Rule::*;

        assert_eq!(Rule::parse_v1("\"a\""), A);
        assert_eq!(Rule::parse_v1("\"b\""), B);
        assert_eq!(Rule::parse_v1("110 61"), Concat(110, 61));
        assert_eq!(Rule::parse_v1("110 61 | 92 103"), Or((110, 61), (92, 103)));
    }

    #[test]
    fn test_rules() {
        let input = "\
            1: 2 3 | 3 2\n\
            3: 4 5 | 5 4\n\
            4: \"a\"\n\
            0: 4 1\n\
            5: \"b\"\n\
            2: 3\
        ";

        let expected = Rules {
            rules: vec![
                Rule::Concat(4, 1),
                Rule::Or((2, 3), (3, 2)),
                Rule::Alias(3),
                Rule::Or((4, 5), (5, 4)),
                Rule::A,
                Rule::B,
            ],
        };
        let actual = Rules::parse_v1(input);

        assert_eq!(actual, expected);
    }
}

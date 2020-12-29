#![allow(clippy::filter_map)]

use anyhow::{anyhow, Context, Result};
use ndarray::Array;
use nom::{
    bytes::complete::{tag, take_until},
    character::complete::{char, digit1, multispace0, newline},
    combinator::{all_consuming, eof, map, map_opt},
    multi::{separated_list0, separated_list1},
    sequence::{preceded, separated_pair, terminated, tuple},
    Finish, IResult,
};
use std::{
    cmp::max,
    fs,
    iter::{self, Iterator, Peekable},
    ops::RangeInclusive,
    str::FromStr,
    u16,
};

type Range = RangeInclusive<u16>;

/// Consume the next item if a condition is true.
fn next_if<T>(
    peekable_iter: &mut Peekable<impl Iterator<Item = T>>,
    func: impl FnOnce(&T) -> bool,
) -> Option<T> {
    match peekable_iter.peek() {
        Some(val) if func(val) => peekable_iter.next(),
        _ => None,
    }
}

#[derive(Debug)]
struct RangeSet {
    merged: Vec<Range>,
}

impl RangeSet {
    fn from_iter(unsorted: impl Iterator<Item = Range>) -> Self {
        // Sort by the start of each range
        let mut sorted = unsorted.collect::<Vec<_>>();
        sorted.sort_unstable_by_key(|range| *range.start());

        let mut sorted = sorted.into_iter().peekable();
        let mut merged = Vec::new();

        while let Some(range) = sorted.next() {
            let (start, mut end) = range.into_inner();

            // "eat" any ranges inside our current range, expanding the end of
            // the current range if their end is larger.
            while let Some(next) = next_if(&mut sorted, |next| *next.start() <= end + 1) {
                end = max(end, *next.end());
            }

            let new_range = Range::new(start, end);
            merged.push(new_range);
        }

        Self { merged }
    }

    fn contains(&self, value: u16) -> bool {
        for range in &self.merged {
            if range.contains(&value) {
                return true;
            }
        }
        false
    }
}

#[derive(Debug, Eq, PartialEq)]
struct Rule<'a> {
    name: &'a str,
    ranges: (Range, Range),
}

impl<'a> Rule<'a> {
    fn is_valid_for(&self, field: u16) -> bool {
        self.ranges.0.contains(&field) || self.ranges.1.contains(&field)
    }
}

#[derive(Debug, Eq, PartialEq)]
struct Ticket {
    fields: Vec<u16>,
}

struct Data<'a> {
    rules: Vec<Rule<'a>>,
    my_ticket: Ticket,
    other_tickets: Vec<Ticket>,
}

fn parse_u16(s: &str) -> IResult<&str, u16> {
    map_opt(digit1, |s| u16::from_str(s).ok())(s)
}

fn parse_range_u16(s: &str) -> IResult<&str, Range> {
    map(
        separated_pair(parse_u16, char('-'), parse_u16),
        |(start, end)| (start..=end),
    )(s)
}

impl<'a> Rule<'a> {
    fn parse(s: &'a str) -> IResult<&'a str, Rule<'a>> {
        let name = take_until(":");
        let ranges = separated_pair(parse_range_u16, tag(" or "), parse_range_u16);
        map(separated_pair(name, tag(": "), ranges), |(name, ranges)| {
            Self { name, ranges }
        })(s)
    }
}

impl Ticket {
    fn new(fields: Vec<u16>) -> Self {
        Self { fields }
    }

    fn parse(s: &str) -> IResult<&str, Self> {
        map(separated_list1(tag(","), parse_u16), Self::new)(s)
    }
}

impl<'a> Data<'a> {
    fn parse(s: &'a str) -> IResult<&'a str, Self> {
        let end = tuple((multispace0, eof));
        let parse_rules = terminated(separated_list1(newline, Rule::parse), tag("\n\n"));
        let parse_my_ticket =
            terminated(preceded(tag("your ticket:\n"), Ticket::parse), tag("\n\n"));
        let parse_other_tickets = terminated(
            preceded(
                tag("nearby tickets:\n"),
                separated_list0(newline, Ticket::parse),
            ),
            end,
        );

        map(
            all_consuming(tuple((parse_rules, parse_my_ticket, parse_other_tickets))),
            |(rules, my_ticket, other_tickets)| Self {
                rules,
                my_ticket,
                other_tickets,
            },
        )(s)
    }
}

fn part1(data: &Data) {
    let ranges = data.rules.iter().flat_map(|rule| {
        let (range1, range2) = rule.ranges.clone();
        iter::once(range1).chain(iter::once(range2))
    });
    let range_set = RangeSet::from_iter(ranges);

    let error_rate: u16 = data
        .other_tickets
        .iter()
        .map(|ticket| {
            ticket
                .fields
                .iter()
                .map(|field| {
                    if range_set.contains(*field) {
                        0
                    } else {
                        *field
                    }
                })
                .sum::<u16>()
        })
        .sum();
    dbg!(error_rate);
}

fn find_rec(
    valid_rules_map: &[(usize, Vec<usize>)],
    current_fields_idx: usize,
    already_chosen_rules: &mut Vec<usize>,
) -> bool {
    if current_fields_idx == valid_rules_map.len() {
        true
    } else {
        for rule_idx in &valid_rules_map[current_fields_idx].1 {
            // skip already chosen rules
            if already_chosen_rules.contains(rule_idx) {
                continue;
            }

            already_chosen_rules.push(*rule_idx);
            let maybe_found = find_rec(
                valid_rules_map,
                current_fields_idx + 1,
                already_chosen_rules,
            );

            if maybe_found {
                return true;
            }

            already_chosen_rules.pop();
        }

        false
    }
}

// recursively search for a satisfying ruleset
fn find_satisfying_ruleset(valid_rules_map: &[(usize, Vec<usize>)]) -> Vec<usize> {
    let current_fields_idx = 0;
    let mut already_chosen_rules = Vec::new();
    find_rec(
        valid_rules_map,
        current_fields_idx,
        &mut already_chosen_rules,
    );

    let mut unshuffled = vec![0_usize; valid_rules_map.len()];
    for (rule_idx, (row_idx, _valid_rules)) in valid_rules_map.iter().enumerate() {
        let chosen_rule = already_chosen_rules[rule_idx];
        unshuffled[*row_idx] = chosen_rule;
    }
    unshuffled
}

fn part2(data: &Data) {
    let num_fields = data.rules.len();

    let ranges = data.rules.iter().flat_map(|rule| {
        let (range1, range2) = rule.ranges.clone();
        iter::once(range1).chain(iter::once(range2))
    });
    let range_set = RangeSet::from_iter(ranges);

    // pull out all the field data as a flat iterator. remove any tickets with
    // invalid fields.
    let elems = data
        .other_tickets
        .iter()
        .filter(|ticket| ticket.fields.iter().all(|field| range_set.contains(*field)))
        .flat_map(|ticket| ticket.fields.iter().copied())
        .collect::<Vec<_>>();
    let num_tickets = elems.len() / num_fields;

    // move the tickets data into a 2d matrix where each row corresponds with an
    // unknown field.
    let tickets = Array::from(elems)
        .into_shape((num_tickets, num_fields))
        .unwrap()
        .reversed_axes();

    // valid_rules_map[i] => { rules valid for all fields in field[i] }
    let mut valid_rules_map = tickets
        .genrows()
        .into_iter()
        .enumerate()
        .map(|(row_idx, row)| {
            let valid_rules = data
                .rules
                .iter()
                .enumerate()
                .filter_map(|(rule_idx, rule)| {
                    if row.iter().all(|field| rule.is_valid_for(*field)) {
                        Some(rule_idx)
                    } else {
                        None
                    }
                });
            (row_idx, valid_rules.collect::<Vec<_>>())
        })
        .collect::<Vec<_>>();

    // sort the fields by # satisfying rules before solving. this makes the solver
    // finish almost instantly.
    valid_rules_map.sort_unstable_by_key(|(_row_idx, valid_rules)| valid_rules.len());

    // find a satisfying ruleset, i.e., a single rule per field and each rule
    // is valid for every entry in that field.
    let satisfying_rules = find_satisfying_ruleset(&valid_rules_map);

    // rule indices with names starting with "departure"
    let departure_rules = data
        .rules
        .iter()
        .enumerate()
        .filter_map(|(idx, rule)| {
            if rule.name.starts_with("departure") {
                Some(idx)
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    // field indices for departure rules
    let departure_fields = satisfying_rules
        .iter()
        .enumerate()
        .filter_map(|(row_idx, rule_idx)| {
            if departure_rules.contains(rule_idx) {
                Some(row_idx)
            } else {
                None
            }
        });

    // my ticket's departure fields
    let my_departure_fields = departure_fields.map(|field_idx| data.my_ticket.fields[field_idx]);

    dbg!(my_departure_fields.map(|num| num as u64).product::<u64>());
}

pub fn run(args: &[&str]) -> Result<()> {
    let input = fs::read_to_string(args[0]).context("Failed to read file")?;

    let (_, data) = Data::parse(&input)
        .finish()
        .map_err(|err| anyhow!("Failed to parse data: {}", err))?;

    time!(part1(&data));
    time!(part2(&data));

    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_parse_rule() {
        let rule_str = "departure location: 26-724 or 743-964";
        let expected = Rule {
            name: "departure location",
            ranges: ((26..=724), (743..=964)),
        };
        assert_eq!(Rule::parse(rule_str), Ok(("", expected)));
    }

    #[test]
    fn test_parse_ticket() {
        let ticket_str =
            "242,255,344,634,710,124,813,241,697,342,600,637,202,421,75,195,496,470,806,554";
        let expected = Ticket {
            fields: vec![
                242, 255, 344, 634, 710, 124, 813, 241, 697, 342, 600, 637, 202, 421, 75, 195, 496,
                470, 806, 554,
            ],
        };
        assert_eq!(Ticket::parse(ticket_str), Ok(("", expected)));
    }

    #[test]
    fn test_range_set() {
        let range_set = RangeSet::from_iter(iter::empty());
        assert_eq!(range_set.merged, &[]);

        let range_set = RangeSet::from_iter(vec![(12..=13), (5..=10)].into_iter());
        assert_eq!(range_set.merged, &[(5..=10), (12..=13)]);

        let range_set = RangeSet::from_iter(vec![(11..=13), (9..=11), (5..=10)].into_iter());
        assert_eq!(range_set.merged, &[(5..=13)]);

        let range_set = RangeSet::from_iter(vec![(12..=13), (9..=11), (5..=10)].into_iter());
        assert_eq!(range_set.merged, &[(5..=13)]);

        let range_set = RangeSet::from_iter(vec![(12..=13), (9..=11), (5..=7)].into_iter());
        assert_eq!(range_set.merged, &[(5..=7), (9..=13)]);

        let range_set = RangeSet::from_iter(vec![(10..=11), (8..=13), (3..=6)].into_iter());
        assert_eq!(range_set.merged, &[(3..=6), (8..=13)]);
    }
}

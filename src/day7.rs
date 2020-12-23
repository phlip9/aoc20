use anyhow::{Context, Result};
use arrayvec::ArrayVec;
use petgraph::{
    data::{Element, FromElements},
    graph::DiGraph,
    visit::{Dfs, DfsPostOrder, EdgeRef, Reversed, Walker},
};
use std::{collections::HashMap, fmt, fs, iter};

struct Rule<'a> {
    bag: &'a str,
    contains: ArrayVec<[(u8, &'a str); 4]>,
}

impl<'a> Rule<'a> {
    fn from_str(s: &'a str) -> Self {
        let mut split = s.split(" bags contain ");

        let bag = split.next().unwrap();
        let rest = split.next().unwrap();
        let rest = rest.trim_end_matches('.');

        let mut contains = ArrayVec::new();

        for bag in rest.split(", ") {
            let bag = bag
                .strip_suffix(" bags")
                .or_else(|| bag.strip_suffix(" bag"))
                .unwrap();

            if bag == "no other" {
                break;
            }

            let idx = bag.find(' ').unwrap();
            let (num, bag) = bag.split_at(idx);

            let num = num.parse::<u8>().unwrap();

            contains.push((num, &bag[1..]));
        }

        Self { bag, contains }
    }
}

impl<'a> fmt::Display for Rule<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} => ", self.bag)?;
        let mut debug_list = f.debug_list();
        for (num, bag) in &self.contains {
            debug_list.entry(&format!("{} {}", num, bag));
        }
        debug_list.finish()
    }
}

impl<'a> fmt::Debug for Rule<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self)
    }
}

#[derive(Debug)]
struct Rules<'a> {
    raw_rules: Vec<Rule<'a>>,
    index_map: HashMap<&'a str, u16>,
    graph: DiGraph<(), u8, u16>,
}

impl<'a> Rules<'a> {
    fn from_raw_rules(raw_rules: Vec<Rule<'a>>) -> Self {
        let num_bags = raw_rules.len();

        let index_map = raw_rules
            .iter()
            .enumerate()
            .map(|(idx, rule)| (rule.bag, idx as u16))
            .collect::<HashMap<_, _>>();

        let nodes = iter::repeat(Element::Node { weight: () }).take(num_bags);
        let edges = raw_rules
            .iter()
            .enumerate()
            .map(|(idx, rule)| {
                let index_map = &index_map;
                let bag_idx = idx as u16;

                rule.contains.iter().map(move |(num, contained_bag)| {
                    let contained_bag_idx = index_map[contained_bag];
                    Element::Edge {
                        source: bag_idx as usize,
                        target: contained_bag_idx as usize,
                        weight: *num,
                    }
                })
            })
            .flatten();
        let elements = nodes.chain(edges);
        let graph = DiGraph::from_elements(elements);

        Self {
            raw_rules,
            index_map,
            graph,
        }
    }

    fn count_containers_of(&self, bag: &'a str) -> usize {
        let bag_idx = self.index_map[bag];
        let count = Dfs::new(&self.graph, bag_idx.into())
            .iter(Reversed(&self.graph))
            .count();
        // Don't include the initial bag
        count - 1
    }

    fn count_contained_of(&self, bag: &'a str) -> u16 {
        // contained_i = sum_{(i,j) in E} w_{i,j} * (1 + contained_j)

        let mut contained = vec![0_u16; self.raw_rules.len()];
        let bag_idx = self.index_map[bag];
        for node in DfsPostOrder::new(&self.graph, bag_idx.into()).iter(&self.graph) {
            let mut sum = 0;
            for edge in self.graph.edges(node) {
                let w_ij = *edge.weight() as u16;
                let contained_j = contained[edge.target().index()];
                sum += w_ij * (1 + contained_j);
            }
            contained[node.index()] = sum;
        }
        contained[bag_idx as usize]
    }
}

pub fn run(args: &[&str]) -> Result<()> {
    let input = fs::read_to_string(args[0]).context("Failed to read file")?;

    let raw_rules = input.lines().map(Rule::from_str).collect::<Vec<_>>();
    let rules = Rules::from_raw_rules(raw_rules);

    dbg!(rules.count_containers_of("shiny gold"));
    dbg!(rules.count_contained_of("shiny gold"));

    Ok(())
}

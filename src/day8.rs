use anyhow::{Context, Result};
use arrayvec::ArrayVec;
use either::Either;
use fixedbitset::FixedBitSet;
use petgraph::{
    data::{Element, FromElements},
    graph::DiGraph,
    visit::{Dfs, Reversed, Walker},
};
use std::{
    fmt, fs,
    iter::{self, ExactSizeIterator},
    ops::Range,
};

type Leaders = FixedBitSet;
type BasicBlock = Range<usize>;
type BasicBlockGraph = DiGraph<(), (), usize>;
type BlockConnectivity = FixedBitSet;

enum Instr {
    Acc(i16),
    Jmp(i16),
    Nop(i16),
}

impl Instr {
    fn from_str(s: &str) -> Self {
        use Instr::*;
        let (instr, val) = s.split_at(3);
        let val = val[1..].parse::<i16>().expect("Invalid value");
        match instr {
            "acc" => Acc(val),
            "jmp" => Jmp(val),
            "nop" => Nop(val),
            _ => panic!("Invalid instruction"),
        }
    }

    const fn is_jmp(&self) -> bool {
        matches!(self, Self::Jmp(_))
    }

    const fn is_nop(&self) -> bool {
        matches!(self, Self::Nop(_))
    }

    fn repair(&mut self) {
        use Instr::*;
        match self {
            Jmp(off) => *self = Nop(*off),
            Nop(off) => *self = Jmp(*off),
            Acc(_) => panic!("Can't repair acc instruction"),
        }
    }
}

impl fmt::Display for Instr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use Instr::*;
        let (instr, val) = match self {
            Acc(val) => ("acc", val),
            Jmp(val) => ("jmp", val),
            Nop(val) => ("nop", val),
        };
        write!(f, "{} {:+}", instr, val)
    }
}

impl fmt::Debug for Instr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self)
    }
}

// Evaluate program instructions, returning Ok(acc) if the program terminates
// and Err(acc) if the program is about to execute an instruction for another
// time (i.e., infinite loop).
fn eval(instrs: &[Instr]) -> Result<i16, i16> {
    let mut visited_instrs = FixedBitSet::with_capacity(instrs.len());

    let mut ip: i16 = 0;
    let mut acc: i16 = 0;
    let terminal_idx = instrs.len() as i16;

    loop {
        // Terminated
        if ip == terminal_idx {
            return Ok(acc);
        }

        let instr = &instrs[ip as usize];
        if visited_instrs[ip as usize] {
            // Hit an already-visited instruction: looping!
            return Err(acc);
        } else {
            visited_instrs.insert(ip as usize);
        }

        // Evaluate instruction
        match instr {
            Instr::Acc(amt) => acc += amt,
            Instr::Jmp(off) => ip += off - 1,
            Instr::Nop(_) => (),
        }
        ip += 1;
    }
}

// Find all basic block leaders
// A leader is:
//   1. the first instruction
//   2. a target of a jmp
//   3. an instruction immediately after a jmp
// include_nop will interpret nops as jmps for the purposes of computing leaders
// (and therefore also basic blocks).
fn leaders(instrs: &[Instr], include_nop: bool) -> Leaders {
    let mut leaders = Leaders::with_capacity(instrs.len());

    for (idx, instr) in instrs.iter().enumerate() {
        let idx = idx as i16;

        // First instruction is a leader
        if idx == 0 {
            leaders.insert(0);
        } else {
            let prev_instr = &instrs[(idx - 1) as usize];

            if prev_instr.is_jmp() || (include_nop && prev_instr.is_nop()) {
                // If previous instruction is a jmp, then we're a leader
                leaders.insert(idx as usize);
            }
        }

        // If we're a jmp, then our target is a leader
        let maybe_target = match instr {
            Instr::Jmp(off) => {
                if idx + off < instrs.len() as i16 {
                    Some(idx + off)
                } else {
                    None
                }
            }
            Instr::Nop(off) if include_nop => {
                if idx + off < instrs.len() as i16 {
                    Some(idx + off)
                } else {
                    None
                }
            }
            _ => None,
        };

        if let Some(target) = maybe_target {
            leaders.insert(target as usize);
        }
    }

    leaders
}

// We can easily compute the basic blocks using the leaders, i.e.,
// basic blocks := { [leader_i, leader_i+1) }_{i in 0..|leaders|}
fn basic_blocks(
    leader_indices: &[usize],
    terminate_idx: usize,
) -> impl Iterator<Item = BasicBlock> + '_ {
    let last_leader_idx = leader_indices[leader_indices.len() - 1];

    leader_indices
        .windows(2)
        .map(|slice| (slice[0]..slice[1]))
        .chain(iter::once(last_leader_idx..terminate_idx))
}

// Build a map from instruction index -> containing basic block index
fn basic_block_map(basic_blocks: &[BasicBlock]) -> impl Iterator<Item = usize> + '_ {
    basic_blocks
        .iter()
        .enumerate()
        .flat_map(|(idx, basic_block)| iter::repeat(idx).take(basic_block.len()))
}

// Build the graph of basic blocks with directed edges connecting them. There are
// two kinds of edges: fallthrough edges, where the previous basic block's end instruction
// is not a jmp (e.g., it's a target of a jmp or a nop), and jmp edges, where the
// end of a basic block is a jmp targeting another basic block.
fn basic_block_graph(
    instrs: &[Instr],
    basic_blocks: &[BasicBlock],
    basic_block_map: &[usize],
) -> BasicBlockGraph {
    let num_blocks = basic_blocks.len();
    let nodes = iter::repeat(Element::Node { weight: () }).take(num_blocks);
    let edges = basic_blocks
        .iter()
        .enumerate()
        .flat_map(|(basic_block_idx, basic_block)| {
            let leader_idx = basic_block.start;
            let end_idx = basic_block.end - 1;

            // 1: fallthrough: prev instr not a jmp: prev bb -> curr bb
            let fallthrough_iter = if leader_idx != 0 && !instrs[leader_idx - 1].is_jmp() {
                // Since we're only iterating over leaders, we don't need to check
                // that the previous instruction is in a different basic block.
                Either::Left(iter::once(Element::Edge {
                    source: basic_block_idx - 1,
                    target: basic_block_idx,
                    weight: (),
                }))
            } else {
                Either::Right(iter::empty())
            };

            // 2: end of basic block is a jmp: curr bb -> target bb
            let jmp_iter = if let Instr::Jmp(off) = &instrs[end_idx] {
                let target_idx = ((end_idx as i16) + *off) as usize;

                if target_idx < instrs.len() {
                    let target_block_idx = basic_block_map[target_idx];
                    Either::Left(iter::once(Element::Edge {
                        source: basic_block_idx,
                        target: target_block_idx,
                        weight: (),
                    }))
                } else {
                    Either::Right(iter::empty())
                }
            } else {
                Either::Right(iter::empty())
            };

            fallthrough_iter.chain(jmp_iter)
        });
    let elements = nodes.chain(edges);
    BasicBlockGraph::from_elements(elements)
}

// Determine which basic blocks are connected to the source (first basic block
// containing the program start instruction). In this case, "connected" means
// executing the program from the beginning will eventually reach this basic block.
//
// Returns a bitset which maps basic block index -> true if that basic block is
// connected to source.
fn source_connectivity(basic_block_graph: &BasicBlockGraph) -> BlockConnectivity {
    let mut connectivity = FixedBitSet::with_capacity(basic_block_graph.node_count());
    let source_idx = 0;

    for node in Dfs::new(&basic_block_graph, source_idx.into()).iter(&basic_block_graph) {
        connectivity.insert(node.index());
    }

    connectivity
}

// Determine which basic blocks are connected to the terminal (last basic block
// containing the program end). In this case, "connected" means if we enter a
// connected basic block, then the program execution will eventually terminate.
//
// Returns a bitset which maps basic block index -> true if that basic block is
// connected to terminal.
fn terminal_connectivity(basic_block_graph: &BasicBlockGraph) -> BlockConnectivity {
    let num_blocks = basic_block_graph.node_count();
    let mut connectivity = FixedBitSet::with_capacity(num_blocks);
    let terminal_idx = num_blocks - 1;

    for node in Dfs::new(&basic_block_graph, terminal_idx.into()).iter(Reversed(&basic_block_graph))
    {
        connectivity.insert(node.index());
    }

    connectivity
}

// Return true if the basic block graph is connected from source -> terminal.
fn is_connected(basic_block_graph: &BasicBlockGraph) -> bool {
    let num_blocks = basic_block_graph.node_count();
    let source_idx = 0;
    let terminal_idx = num_blocks - 1;

    for node in Dfs::new(&basic_block_graph, source_idx.into()).iter(&basic_block_graph) {
        if node.index() == terminal_idx {
            return true;
        }
    }

    false
}

// Find the single jmp or nop instruction that when "repaired" will allow the
// program to terminate.
//
// Strategy:
//
//  1. find leaders
//  2. basic blocks from leaders
//  3. basic block graph
//  4. source connectivity
//  5. terminal connectivity
//  6. walk source-connected basic block graph to find repair that connects
//     terminal-connected basic block graph.
fn find_repair(instrs: &[Instr]) -> Option<usize> {
    let include_nop = true;
    let leaders = leaders(instrs, include_nop);
    let leader_indices = leaders.ones().collect::<Vec<_>>();

    let terminal_idx = instrs.len();
    let basic_blocks = basic_blocks(&leader_indices, terminal_idx).collect::<Vec<_>>();
    let basic_block_map = basic_block_map(&basic_blocks).collect::<Vec<_>>();
    let basic_block_graph = basic_block_graph(instrs, &basic_blocks, &basic_block_map);

    // Already connected; no repair needed.
    if is_connected(&basic_block_graph) {
        return None;
    }

    let source_connectivity = source_connectivity(&basic_block_graph);
    let terminal_connectivity = terminal_connectivity(&basic_block_graph);

    // Objective: Find a leader or exit instruction in a source-connected basic
    // block that, when "repaired", will connect source -> terminal.
    //
    // We only need to check adding edges since removing edges can never improve
    // connectivity from source -> terminal.

    let source_blocks_and_instrs = source_connectivity.ones().flat_map(|block_idx| {
        let block = &basic_blocks[block_idx];
        let leader_idx = block.start;
        let end_idx = block.end - 1;

        let mut instr_idxs = ArrayVec::<[(usize, usize); 2]>::new();
        instr_idxs.push((block_idx, leader_idx));

        if leader_idx != end_idx {
            instr_idxs.push((block_idx, end_idx));
        }

        instr_idxs
    });

    for (block_idx, instr_idx) in source_blocks_and_instrs {
        match &instrs[instr_idx] {
            Instr::Jmp(_) => {
                // Jmp -> Nop
                // Jmp is always the end of a basic block
                //   ==> Add fallthrough edge
                //   ==> Remove jmp edge : cannot improve connectivity

                // Adding the fallthrough edge connects source -> terminal: we're done!
                // source_connectivity.contains(block_idx) is implied
                if terminal_connectivity.contains(block_idx + 1) {
                    return Some(instr_idx);
                }
            }
            Instr::Nop(off) => {
                // Nop -> Jmp
                //   ==> Add jmp edge
                // If fallthrough source
                //   ==> Remove fallthrough edge : cannot improve connectivity

                let target_idx = ((instr_idx as i16) + *off) as usize;
                let target_block_idx = basic_block_map[target_idx];

                // Adding this edge connects source -> terminal: we're done!
                // source_connectivity.contains(block_idx) is implied
                if terminal_connectivity.contains(target_block_idx) {
                    return Some(instr_idx);
                }
            }
            Instr::Acc(_) => (),
        }
    }

    Some(0)
}

fn parse_instructions(program: &str) -> Vec<Instr> {
    program.lines().map(Instr::from_str).collect::<Vec<_>>()
}

pub fn run(args: &[&str]) -> Result<()> {
    let input = fs::read_to_string(args[0]).context("Failed to read file")?;
    let mut instrs = parse_instructions(&input);

    // part 1
    dbg!(eval(&instrs).expect_err("Part 1 should loop"));

    // part 2
    let repair_instr_idx = dbg!(find_repair(&instrs)).expect("Should be a repair");
    instrs[repair_instr_idx].repair();

    dbg!(eval(&instrs)).expect("Should terminate after repair");

    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;
    use petgraph::visit::EdgeRef;

    // Test on a small sample program
    #[test]
    fn test_repair() {
        let program = "\
            nop +0\n\
            acc +1\n\
            jmp +4\n\
            acc +3\n\
            jmp -3\n\
            acc -99\n\
            acc +1\n\
            jmp -4\n\
            jmp +1\
        ";

        let mut instrs = parse_instructions(program);
        assert_eq!(eval(&instrs), Err(5));

        let leaders = leaders(&instrs, false);

        let indices = leaders.ones().collect::<Vec<_>>();
        assert_eq!(&[0, 1, 3, 5, 6, 8][..], &indices[..]);

        let basic_blocks = basic_blocks(&indices, instrs.len()).collect::<Vec<_>>();
        assert_eq!(
            &basic_blocks[..],
            &[(0..1), (1..3), (3..5), (5..6), (6..8), (8..9)][..],
        );

        let basic_block_map = basic_block_map(&basic_blocks).collect::<Vec<_>>();
        assert_eq!(&basic_block_map[..], &[0, 1, 1, 2, 2, 3, 4, 4, 5][..]);

        let basic_block_graph = basic_block_graph(&instrs, &basic_blocks, &basic_block_map);

        let mut edges = basic_block_graph
            .edge_references()
            .map(|edge| (edge.source().index(), edge.target().index()))
            .collect::<Vec<_>>();
        edges.sort_unstable();

        assert_eq!(&edges[..], &[(0, 1), (1, 4), (2, 1), (3, 4), (4, 2)][..]);

        let source_connectivity = source_connectivity(&basic_block_graph);
        let mut src_conn_idxs = source_connectivity.ones().collect::<Vec<_>>();
        src_conn_idxs.sort_unstable();
        assert_eq!(&src_conn_idxs[..], &[0, 1, 2, 4][..]);

        let terminal_connectivity = terminal_connectivity(&basic_block_graph);
        let term_conn_idxs = terminal_connectivity.ones().collect::<Vec<_>>();
        assert_eq!(&term_conn_idxs[..], &[5][..]);

        let repair_instr = find_repair(&instrs);
        assert_eq!(repair_instr, Some(7));

        instrs[repair_instr.unwrap()].repair();
        assert_eq!(eval(&instrs), Ok(2));
    }
}

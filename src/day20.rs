#![allow(unused_imports)]

// SOLUTION CODE GOES HERE

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum Cell {
    Wall,
    Open,
    Label(u8),
}

// We assume that gates are uniquely identified by their unordered pair of
// letters
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
struct Gate {
    g1: u8,
    g2: u8,
}

impl Gate {
    fn new(g1: u8, g2: u8) -> Self {
        Self {
            g1: g1.min(g2),
            g2: g1.max(g2),
        }
    }
}

use Cell::*;

// Remove if this is not an intcode problem
#[inline(always)]
fn solve(input: &str) -> impl Display {
    let mut grid: SparseGrid<Cell> = SparseGrid::new_default((0, 0), Wall);

    eprintln!("Parsing input...");
    for (row, line) in input.lines().enumerate() {
        let row = Row(row as isize);
        for (column, cell) in line.trim_end().as_bytes().iter().enumerate() {
            let column = Column(column as isize);
            match *cell {
                b' ' | b'#' => {}
                b'.' => {
                    grid.insert((row, column), Open);
                }
                gate @ b'A'..=b'Z' => {
                    grid.insert((row, column), Label(gate));
                }
                cell => panic!("Unexpected cell value '{}' at {:?}", cell, (row, column)),
            }
        }
    }

    eprintln!("Finding portals...");
    // Parse the gates
    let mut gates: HashMap<Location, Location> = HashMap::default();
    let mut partial_gates: HashMap<Gate, Location> = HashMap::default();

    for row in grid.rows().iter() {
        for (location, cell) in row.iter_with_locations() {
            if let &Label(g1) = cell {
                for &direction in &EACH_DIRECTION {
                    let g2_loc = location.step(direction);
                    let gate_loc = g2_loc.step(direction);

                    if let Ok(&Label(g2)) = grid.get(&g2_loc) {
                        if let Ok(&Open) = grid.get(&gate_loc) {
                            let gate = Gate::new(g1, g2);
                            match partial_gates.entry(gate) {
                                Entry::Vacant(vacancy) => {
                                    vacancy.insert(gate_loc);
                                }
                                Entry::Occupied(occupied) => {
                                    let gate_other_loc = occupied.remove();

                                    gates.insert(gate_loc, gate_other_loc);
                                    gates.insert(gate_other_loc, gate_loc);
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    assert_eq!(partial_gates.len(), 2);
    let &start_loc = partial_gates.get(&Gate::new(b'A', b'A')).unwrap();
    let &end_loc = partial_gates.get(&Gate::new(b'Z', b'Z')).unwrap();

    eprintln!("Searching the maze...");
    let mut searchers: HashSet<Location> = HashSet::from_iter(Some(start_loc));
    let mut next_searchers: HashSet<Location> = HashSet::new();

    for steps in 1.. {
        for &search_loc in &searchers {
            let adjacent_targets = EACH_DIRECTION.iter().map(|d| search_loc + d);
            let portal_target = gates.get(&search_loc).copied();
            let targets = adjacent_targets.chain(portal_target);

            for target in targets {
                if target == end_loc {
                    return steps;
                }

                if let Ok(Open) = grid.get(&target) {
                    grid.set(target, Wall).unwrap();
                    next_searchers.insert(target);
                }
            }
        }
        swap(&mut searchers, &mut next_searchers);
        next_searchers.clear();
    }

    panic!("No solution found!")
}

/*
 * SUPPORTING LIBRARY CODE GOES HERE:
 *
 * - Imports & use statements for tons of common traits, data structures, etc
 * - `fn main` bootstrap that reads from stdin and writes the solution to stdout
 * - Utility traits
 * - Anything else that might be broadly useful for other problems
 */

#[global_allocator]
static ALLOC: jemallocator::Jemalloc = jemallocator::Jemalloc;

use std::cmp::{Eq, Ord, Ordering, PartialEq, PartialOrd};
use std::collections::hash_map::{Entry, OccupiedEntry, VacantEntry};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::error::Error;
use std::fmt::{self, Display, Formatter, Write as FmtWrite};
use std::hash::Hash;
use std::io::{self, Read, Write as IoWrite};
use std::iter::{self, FromIterator, FusedIterator, Peekable};
use std::mem::{replace, swap};
use std::ops::Add;
use std::rc::{Rc, Weak};
use std::str::FromStr;
use std::thread::sleep;
use std::time::{Duration, Instant};

// String joins
use joinery::prelude::*;

// Grids
use gridly::prelude::*;
use gridly_grids::*;

// Generation-based simulations
use generations::*;

// Formatting things without creating intermediary strings
use lazy_format::lazy_format;

// Cascading init
use cascade::cascade;

// Integer traits
use num::Integer;

// Parsing
use nom::{
    bytes::complete::tag,
    character::complete::{alpha1, digit1, multispace0, multispace1, space0, space1},
    combinator::{all_consuming, iterator, map, map_res, opt, recognize},
    multi::{many0, separated_list},
    sequence::{delimited, pair, preceded, separated_pair, terminated, tuple},
    IResult,
};

#[inline(always)]
fn timed<T>(f: impl FnOnce() -> T) -> (T, Duration) {
    let start = Instant::now();
    let result = f();
    let end = Instant::now();
    (result, end - start)
}

fn main() {
    let mut input = String::with_capacity(4096);
    io::stdin()
        .read_to_string(&mut input)
        .unwrap_or_else(|err| panic!("Error reading input from stdin: {}", err));
    let (solution, duration) = timed(move || solve(&input));
    println!("{}", solution);
    eprintln!("Algorithm duration: {:?}", duration);
}

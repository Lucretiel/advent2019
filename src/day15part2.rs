#![allow(unused_imports)]

// SOLUTION CODE GOES HERE

// Remove if this is not an intcode problem
mod intcode;
use intcode::*;

fn as_command(d: Direction) -> isize {
    match d {
        Up => 1,
        Down => 2,
        Left => 3,
        Right => 4,
    }
}

#[derive(Debug, Clone, Default)]
struct Searcher {
    machine: Machine,
    location: Location,
}

#[derive(Debug, Clone, Default)]
struct Pool<T: Clone> {
    pool: Vec<T>,
}

impl<T: Clone> Pool<T> {
    fn clone_instance(&mut self, target: &T) -> T {
        match self.pool.pop() {
            Some(mut instance) => {
                instance.clone_from(target);
                instance
            }
            None => target.clone(),
        }
    }

    fn add_instance(&mut self, instance: T) {
        self.pool.push(instance);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Cell {
    Unknown,
    Wall,
    Open,
    Oxygen,
}

use Cell::*;

#[inline(always)]
fn solve(input: &str) -> impl Display {
    // Explosive forking tree search. Assumes no loops.
    let machine = Machine::from_csv(input);
    let mut machine_pool: Pool<Machine> = Pool::default();

    let mut grid = SparseGrid::new_default((0, 0), Unknown);
    grid.insert(Location::zero(), Open);

    // Start with a single searcher
    let mut searchers = vec![Searcher {
        machine: machine,
        location: Location::zero(),
    }];
    let mut next_searchers = vec![];

    let mut oxygens = vec![];
    let mut next_oxygens = vec![];

    while !searchers.is_empty() {
        for searcher in searchers.drain(..) {
            for &direction in &EACH_DIRECTION {
                // Don't go into known locations
                let dest = searcher.location + direction;
                match grid.get(dest) {
                    Ok(&Wall) | Ok(&Open) | Ok(&Oxygen) => continue,
                    _ => {}
                }

                // Clone this machine
                let mut child_machine = machine_pool.clone_instance(&searcher.machine);

                // Attempt a move
                match feed(as_command(direction))(&mut child_machine) {
                    MachineState::Halt => panic!("Unexpected halt!"),
                    MachineState::NeedInput => panic!("Unexpected need more input!"),
                    MachineState::Output(code) => match code {
                        // Wall
                        0 => {
                            // Put the machine back
                            machine_pool.add_instance(child_machine);
                            // Add this wall to the list
                            grid.insert(dest, Wall);
                        }
                        // Hallway, oxygen
                        code @ 1 | code @ 2 => {
                            // Keep this searcher for the next iteration
                            next_searchers.push(Searcher {
                                machine: child_machine,
                                location: dest,
                            });

                            match code {
                                1 => {
                                    grid.insert(dest, Open);
                                }
                                2 => {
                                    grid.insert(dest, Oxygen);
                                    oxygens.push(dest);
                                }
                                _ => unreachable!(),
                            }
                        }
                        code => panic!("Invalid code returned: {}", code),
                    },
                }
            }

            // We're done with this searcher. Add its machine to the pool.
            machine_pool.add_instance(searcher.machine);
        }

        swap(&mut searchers, &mut next_searchers);
    }

    let mut steps = 0;

    while !oxygens.is_empty() {
        for oxygen in oxygens.drain(..) {
            for &direction in &EACH_DIRECTION {
                let target = oxygen + direction;
                if let Ok(Open) = grid.get(target) {
                    grid.set(target, Oxygen).unwrap();
                    next_oxygens.push(target);
                }
            }
        }

        steps += 1;
        swap(&mut oxygens, &mut next_oxygens);
    }

    // The last step doesn't count cause no new oxygen was created
    steps - 1
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

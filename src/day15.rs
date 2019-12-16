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

    // The last step this searcher took. Don't go backwards.
    direction: Option<Direction>,
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

#[inline(always)]
fn solve(input: &str) -> impl Display {
    // Explosive forking tree search. Assumes no loops.
    let machine = Machine::from_csv(input);
    let mut machine_pool: Pool<Machine> = Pool::default();

    let mut known_walls: HashSet<Location> = HashSet::default();

    // Start with a single searcher
    let mut searchers = vec![Searcher {
        machine: machine,
        location: Location::zero(),
        direction: None,
    }];
    let mut next_searchers = Vec::new();

    for num_steps in 1u64.. {
        if searchers.is_empty() {
            panic!("Ran out of search space, no oxygen found");
        }

        for searcher in searchers.drain(..) {
            for &direction in &EACH_DIRECTION {
                // Don't go backwards
                if Some(direction.reverse()) == searcher.direction {
                    continue;
                }

                // Don't go into known walls
                let dest = searcher.location + direction;
                if known_walls.contains(&dest) {
                    continue;
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
                            known_walls.insert(dest);
                        }
                        // Hallway
                        1 => {
                            next_searchers.push(Searcher {
                                machine: child_machine,
                                location: dest,
                                direction: Some(direction),
                            });
                        }
                        // Oxygen!
                        2 => {
                            eprintln!("Found at {:?}", dest);
                            return num_steps;
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

    panic!("NO SOLUTION FOUND")
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

use std::cell::Cell;
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

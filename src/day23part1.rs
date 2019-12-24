#![allow(unused_imports)]

// SOLUTION CODE GOES HERE

// Remove if this is not an intcode problem
mod intcode;
use intcode::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Packet {
    dest: usize,
    value: isize,
}

#[inline(always)]
fn solve(input: &str) -> impl Display {
    let init = Machine::from_csv(input);

    let mut machines: Vec<Machine> = iter::repeat_with(|| init.clone()).take(50).collect();

    // We need to initialize the machines with their addresses
    let mut queue: VecDeque<Packet> = (0..50usize)
        .map(|addr| Packet {
            dest: addr,
            value: addr as isize,
        })
        .collect();

    loop {
        // Two phase iteration cycle: deliver packets, then (when the queue is
        // totally empty) send -1 to each machine.
        while let Some(packet) = queue.pop_front() {
            // eprintln!("machine {} receives {}", packet.dest, packet.value);

            let machine = &mut machines[packet.dest];
            let mut stepper = run_until_block(Some(packet.value));

            loop {
                match stepper(machine) {
                    MachineState::Halt => panic!("Unexpected halt!"),
                    MachineState::NeedInput => break,
                    MachineState::Output(dest) => {
                        let dest = dest as usize;
                        // We're getting a packet. The first value is the
                        // address, then we get X and Y (which we deliver as
                        // two separate packets)
                        let x = stepper(machine).expect_out("reading packet x");
                        let y = stepper(machine).expect_out("reading packet y");

                        if dest == 255 {
                            return y;
                        }

                        queue.reserve(2);
                        queue.push_back(Packet { dest, value: x });
                        queue.push_back(Packet { dest, value: y });
                        // eprintln!("machine {} sends ({}, {}) to {}", packet.dest, x, y, dest);
                    }
                }
            }
        }

        // At this point, all machines are blocked on input
        queue.extend((0..50usize).map(|addr| Packet {
            dest: addr,
            value: -1,
        }));
    }
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
use std::collections::{BTreeMap, HashMap, HashSet, VecDeque};
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

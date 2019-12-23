#![allow(unused_imports)]

// SOLUTION CODE GOES HERE

// Remove if this is not an intcode problem
mod intcode;
use intcode::*;

fn main() {
    let stdin = io::stdin();
    let mut stdin_lock = stdin.lock();
    let mut input = String::with_capacity(4096);

    stdin_lock
        .read_line(&mut input)
        .unwrap_or_else(|err| panic!("Error reading input from stdin: {}", err));
    let mut machine = Machine::from_csv(&input);

    let input_bytes = stdin_lock.bytes().map(|b| match b {
        Ok(b) => b as isize,
        Err(err) => panic!("Error reading input from stdin: {}", err),
    });

    let mut stepper = run_until_block(input_bytes);

    let stdout = io::stdout();
    let stdout_lock = stdout.lock();
    let mut buffered = LineWriter::new(stdout_lock);

    loop {
        match stepper(&mut machine) {
            MachineState::Halt => break,
            MachineState::NeedInput => panic!("Unexpected unfullfilled input request"),
            MachineState::Output(output) => {
                if output > 127 {
                    writeln!(buffered, "damage: {}", output).unwrap();
                } else {
                    let byte = output as u8;
                    buffered.write_all(&[byte]).unwrap();
                }
            }
        }
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
use std::collections::{BTreeMap, HashMap, HashSet};
use std::error::Error;
use std::fmt::{self, Display, Formatter, Write as FmtWrite};
use std::hash::Hash;
use std::io::{self, BufRead, LineWriter, Read, Write as IoWrite};
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

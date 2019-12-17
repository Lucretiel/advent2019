#![allow(unused_imports)]

// SOLUTION CODE GOES HERE

// Remove if this is not an intcode problem
mod intcode;
use intcode::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Cell {
    Empty,
    Scaffold,
    Rb(Robot),
}

use Cell::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Robot(Direction);

#[inline(always)]
fn solve(input: &str) -> impl Display {
    let mut machine = Machine::from_csv(input);

    let mut camera_view = String::new();

    let mut stepper = run_until_block(None);

    loop {
        match stepper(&mut machine) {
            MachineState::NeedInput => break,
            MachineState::Halt => break,
            MachineState::Output(byte) => camera_view.push((byte as u8) as char),
        }
    }

    let rows = camera_view.trim_matches('\n').lines().map(|line| {
        line.trim_matches('\n').as_bytes().iter().map(|&c| match c {
            b'#' => Scaffold,
            b'.' => Empty,
            b'^' => Rb(Robot(Up)),
            b'v' => Rb(Robot(Down)),
            b'<' => Rb(Robot(Left)),
            b'>' => Rb(Robot(Right)),
            _ => panic!("Invalid cell: {}", c),
        })
    });

    let mut grid = SparseGrid::new_default((0, 0), Cell::Empty);

    for (r, row) in rows.enumerate() {
        for (c, cell) in row.enumerate() {
            grid.insert(Row(r as isize) + Column(c as isize), cell);
        }
    }

    grid.occuppied_entries()
        // Look at only the scaffolds
        .filter(|(_, &cell)| cell == Scaffold)
        // Find the ones that are intersections
        .filter(|(&loc, _)| {
            EACH_DIRECTION
                .iter()
                .all(|dir| grid.get(loc + dir) == Ok(&Scaffold))
        })
        // Find their alignment parameters
        .map(|(&loc, _)| loc.row.0 * loc.column.0)
        .sum::<isize>()
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

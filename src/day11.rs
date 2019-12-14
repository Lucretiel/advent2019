#![allow(unused_imports)]

// SOLUTION CODE GOES HERE

// Remove if this is not an intcode problem
mod intcode;
use intcode::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Color {
    Black,
    White,
}

impl Color {
    fn color_value(&self) -> isize {
        match self {
            Color::Black => 0,
            Color::White => 1,
        }
    }

    fn from_value(value: isize) -> Color {
        match value {
            0 => Color::Black,
            1 => Color::White,
            _ => panic!("Invalid color {}", value),
        }
    }
}

impl Display for Color {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Color::Black => ' '.fmt(f),
            Color::White => '█'.fmt(f),
        }
    }
}

fn print<T: Display>(mut dest: impl io::Write, grid: &impl Grid<Item = T>) -> io::Result<()> {
    for row in grid.rows().iter() {
        for cell in row.iter() {
            write!(dest, "{}", cell)?;
        }
        write!(dest, "\n")?;
    }
    writeln!(dest, "------------------------------------------------")
}

#[inline(always)]
fn solve(input: &str) -> impl Display {
    let mut machine = Machine::from_csv(input);

    let mut panel = SparseGrid::new_default((0, 0), Color::Black);
    panel.insert(&(0, 0), Color::White);

    let mut robot_location = Location::zero();
    let mut robot_direction = Direction::Up;

    loop {
        let camera = iter::repeat(
            panel
                .get(&robot_location)
                .unwrap_or(&Color::Black)
                .color_value(),
        );
        let mut exec = run_until_block(camera);

        // Step the robot. Feed it the current color until it outputs something.
        let paint = match exec(&mut machine) {
            MachineState::NeedInput => unreachable!(),
            MachineState::Halt => break,
            MachineState::Output(value) => Color::from_value(value),
        };

        robot_direction = match exec(&mut machine) {
            MachineState::NeedInput => unreachable!(),
            MachineState::Halt => panic!("Unexpected halt between paint and turn"),
            MachineState::Output(turn) => match turn {
                0 => robot_direction.anticlockwise(),
                1 => robot_direction.clockwise(),
                _ => panic!("Invalid rotatation: {}", turn),
            },
        };

        panel.insert(robot_location, paint);
        robot_location = robot_location.step(robot_direction);
    }

    print(io::stderr().lock(), &panel).unwrap();

    " "
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

use lazy_static::lazy_static;
use regex::{self, Regex};

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

/// This trait provides methods for extrating fields from a parsed regex. They
/// assume that a match's groups are present, and panic if not.
trait RegexExtractor<'t> {
    fn field(&self, index: usize) -> &'t str;

    fn parse<T: FromStr>(&self, index: usize) -> T
    where
        T::Err: Display;
}

impl<'t> RegexExtractor<'t> for regex::Captures<'t> {
    #[inline]
    fn field(&self, index: usize) -> &'t str {
        self.get(index)
            .unwrap_or_else(move || panic!("Group {} didn't match anything", index))
            .as_str()
    }

    #[inline]
    fn parse<T: FromStr>(&self, index: usize) -> T
    where
        T::Err: Display,
    {
        let field = self.field(index);

        field.parse().unwrap_or_else(move |err| {
            panic!("Failed to parse group {} \"{}\": {}", index, field, err)
        })
    }
}

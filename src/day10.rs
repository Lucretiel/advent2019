#![allow(unused_imports)]

// SOLUTION CODE GOES HERE

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct LineOfSight(Vector);

impl LineOfSight {
    fn new(value: Vector) -> Self {
        Self(reduce(value))
    }

    fn angle(&self) -> f64 {
        f64::atan2(-self.0.columns.0 as f64, self.0.rows.0 as f64)
    }
}

impl PartialOrd for LineOfSight {
    fn partial_cmp(&self, rhs: &LineOfSight) -> Option<Ordering> {
        Some(self.cmp(rhs))
    }
}

impl Ord for LineOfSight {
    fn cmp(&self, rhs: &LineOfSight) -> Ordering {
        self.angle().partial_cmp(&rhs.angle()).unwrap()
    }
}

#[inline]
fn reduce(vector: Vector) -> Vector {
    let factor = vector.rows.0.gcd(&vector.columns.0);

    Vector::new(vector.rows.0 / factor, vector.columns.0 / factor)
}

fn print<T: Display>(grid: &impl Grid<Item = T>) {
    for row in grid.rows().iter() {
        for cell in row.iter() {
            eprint!("{}", cell);
        }
        eprintln!("");
    }
    eprintln!("------------------------------------------");
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Cell {
    Empty,
    Base,
    Asteroid,
    Vaporized(usize),
}

impl Cell {
    fn vaporize(&mut self) {
        if let Cell::Vaporized(..) = *self {
            *self = Cell::Empty;
        }
    }
}

impl Display for Cell {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Cell::Empty => ' '.fmt(f),
            Cell::Base => 'B'.fmt(f),
            Cell::Asteroid => '*'.fmt(f),
            Cell::Vaporized(n) => n.fmt(f),
        }
    }
}

#[inline(always)]
fn solve(input: &str) -> impl Display {
    let base_location = Location::new(19, 27);

    let mut visibility_chart: BTreeMap<LineOfSight, Vec<Location>> = BTreeMap::default();
    let mut grid = SparseGrid::new_default((33, 33), Cell::Empty);
    grid[base_location] = Cell::Base;

    // Build the visibility_chart
    for (row_index, row) in input.trim().lines().enumerate() {
        for (column_index, cell) in row.trim().bytes().enumerate() {
            let target = Row(row_index as isize) + Column(column_index as isize);

            if target != base_location && cell == b'#' {
                let los = LineOfSight::new(target - base_location);

                visibility_chart.entry(los).or_default().push(target);
                grid[target] = Cell::Asteroid;
            }
        }
    }

    print(&grid);

    // Sort each visible asteroid by distance
    visibility_chart.values_mut().for_each(|targets| {
        targets.sort_unstable_by_key(|target| (*target - base_location).manhattan_length());
    });

    let mut count = 0;

    loop {
        for targets in visibility_chart.values_mut().filter(|t| !t.is_empty()) {
            let vaporized = targets.remove(0);
            if count == 198 {
                return lazy_format!("{:?}", vaporized);
            } else {
                grid[vaporized] = Cell::Vaporized(count % 10);
                if count % 10 == 9 {
                    print(&grid);

                    grid.occuppied_entries_mut()
                        .for_each(|(_, cell)| cell.vaporize());
                    grid.clean();
                }
            }
            count += 1;
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

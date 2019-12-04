#![allow(unused_imports)]

// SOLUTION CODE GOES HERE

// Err means there was an overflow
type OverflowResult = Result<(), ()>;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Default, PartialOrd, Ord)]
struct Digit(u8);

impl Digit {
    /// increment the digit; return true if it overflowed
    fn inc(&mut self) -> OverflowResult {
        match self.0 {
            9 => Err(()),
            value => {
                self.0 = value + 1;
                Ok(())
            }
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Default, PartialOrd, Ord)]
struct Password([Digit; 6]);

impl Password {
    fn to_int(&self) -> usize {
        // idk what endian this is, but self[0] is our most significant digit
        self.0
            .iter()
            .copied()
            .fold(0, |accum, digit| (accum * 10) + (digit.0 as usize))
    }

    fn from_int(value: usize) -> Self {
        let mut res = Self::default();
        let mut buf: [u8; 6] = [0; 6];
        write!(&mut buf[..], "{:06}", value).expect("Failed to convert int to password");
        for i in 0..6 {
            //Ascii 0 is 0x30, aka 48
            assert!(48 <= buf[i] && buf[i] < 58);
            res.0[i] = Digit(buf[i] - 48);
        }
        res
    }

    // Increment the given digit, and constrain for the non-increasing
    // requirement.
    fn inc_at(&mut self, index: usize) -> OverflowResult {
        match self.0[index].inc() {
            Ok(()) => {
                for i in (index+1)..6 {
                    self.0[i] = self.0[index];
                }
                Ok(())
            }
            Err(()) if index == 0 => Err(()),
            Err(()) => self.inc_at(index - 1),
        }
    }

    // Check if there is at least one run of exactly 2 of the same digit in
    // the password
    fn is_constrained_part_2(&self) -> bool {
        let mut run_value = self.0[0];
        let mut run_length = 1;
        for &digit in &self.0[1..6] {
            if digit == run_value {
                run_length += 1;
            } else if run_length == 2 {
                return true
            } else {
                run_value = digit;
                run_length = 1;
            }
        }
        false
    }

    // ensure that a pair exists somewhere by incrementing. May do nothing;
    // make sure to increment before using this. returns true on overflow.
    fn constrain_pair(&mut self) -> OverflowResult {
        while !self.is_constrained_part_2() {
            // It's possible that the value is 123444, and 123445 is valid, so
            // increment the last digit
            self.inc_at(5)?;
        }
        Ok(())
    }
}

impl PartialEq<usize> for Password {
    fn eq(&self, rhs: &usize) -> bool {
        self.to_int().eq(rhs)
    }
}

impl PartialOrd<usize> for Password {
    fn partial_cmp(&self, rhs: &usize) -> Option<Ordering> {
        self.to_int().partial_cmp(rhs)
    }
}

fn main() {
    let mut password = Password::from_int(277788);
    let mut count = 0;
    while password < 815432 {
        count += 1;
        println!("{:?}", password.to_int());
        password.inc_at(5).expect("Overflow!");
        password.constrain_pair().expect("Overflow!");
    }
    eprintln!("{}", count)
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
use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::fmt::{self, Display, Formatter};
use std::hash::Hash;
use std::io::{self, Read, Write};
use std::iter::{self, FromIterator, FusedIterator, Peekable};
use std::mem::{replace, swap};
use std::ops::Add;
use std::rc::{Rc, Weak};
use std::str::FromStr;
use std::thread::sleep;
use std::time::{Duration, Instant};



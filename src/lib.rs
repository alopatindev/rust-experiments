#![allow(unstable_features)]
#![allow(zero_prefixed_literal)] // FIXME
#![feature(io)]
#![feature(plugin)]
#![plugin(clippy)]
#![plugin(quickcheck_macros)]

#![feature(test)]
extern crate test;

#[cfg(test)]
extern crate quickcheck;

#[macro_use]
extern crate maplit;

extern crate byteorder;

pub mod algorithms;
pub mod cli;
pub mod encoding;
pub mod format;
pub mod structs;
pub mod terminal;

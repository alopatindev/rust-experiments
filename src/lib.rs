#![allow(unstable_features)]
#![allow(zero_prefixed_literal)] // FIXME
#![feature(io)]
#![cfg_attr(feature="clippy", feature(plugin))]
#![cfg_attr(feature="clippy", plugin(clippy))]

#![feature(test)]
extern crate test;

#[cfg(test)]
#[macro_use]
extern crate quickcheck;

#[macro_use]
extern crate maplit;

extern crate byteorder;

extern crate nalgebra;

extern crate rand;

pub mod algorithms;
pub mod cli;
pub mod encoding;
pub mod format;
pub mod structs;
pub mod terminal;

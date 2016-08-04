#![feature(test)]
extern crate test;

#[cfg(test)]
#[macro_use]
extern crate quickcheck;

pub mod algorithms;
pub mod cli;
pub mod encoding;
pub mod structs;

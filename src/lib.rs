#[macro_use]
extern crate failure;
extern crate byteorder;
#[cfg(test)]
extern crate glob;

pub mod cpu;
pub mod errors;
pub mod rom;

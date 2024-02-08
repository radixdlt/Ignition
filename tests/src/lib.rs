//! The test files only contain tests while any functionality that they all need
//! to have in common is implemented in this library.

#![warn(clippy::arithmetic_side_effects)]

mod environment;
mod errors;
mod extensions;
mod indexed_buckets;

pub mod prelude;

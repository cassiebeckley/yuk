//! Embeddable JavaScript interpreter.

#[macro_use]
extern crate maplit;

pub mod parser;

pub mod ast;
pub mod interpret;

pub mod runtime;

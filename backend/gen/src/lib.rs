//! buf generate の出力(src/gen)を公開するだけの crate。手で書くコードは置かない
#![allow(clippy::all, clippy::pedantic)]

#[path = "gen/mod.rs"]
mod generated;

pub use generated::*;

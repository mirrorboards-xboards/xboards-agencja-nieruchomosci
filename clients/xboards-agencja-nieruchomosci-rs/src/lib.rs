//! The XMeta catalogs of an XBoard in Rust: a module per position with a catalog, e.g.
//! `cases::sprawa`, and in it a module per kind of its room. Written by xmeta-rust-codegen
//! from the description of the catalogs; `xmeta codegen` keeps it, so it is not edited.

pub use xmeta_room;

#[rustfmt::skip]
pub mod cases;
#[rustfmt::skip]
pub mod positions;
#[rustfmt::skip]
pub mod registers;

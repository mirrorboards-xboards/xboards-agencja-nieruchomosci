//! The scenarios of the agency's catalogs. Each module checks one room on SQLite in memory: the
//! flow of the profile of its position and the wrong states its database refuses. The room and
//! its operations are the generated Rust client, so a scenario calls the operations the app calls.
//!
//! Some rules of the tables guard the data against writers outside the client: another app,
//! hand-written SQL or a sync. Such a writer can leave out a value an operation requires or give
//! a value outside a list. The typed operations cannot make that write, so a scenario writes it
//! itself with `MemoryRoom::execute` and expects the table to refuse it.

#[cfg(test)]
mod expect;

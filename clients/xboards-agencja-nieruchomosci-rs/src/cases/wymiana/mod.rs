//! The room of the position `cases/wymiana`: a module per kind, a kind after the kinds it requires.

pub mod wymiana_metryka;
pub mod wymiana_wpisy;
pub mod wymiana_akt;
pub mod wymiana_dokumenty;
pub mod wymiana_dane;

/// The kinds of the room, a kind after the kinds it requires.
pub const KINDS: &[&xmeta_room::Kind] = &[
    &wymiana_metryka::KIND,
    &wymiana_wpisy::KIND,
    &wymiana_akt::KIND,
    &wymiana_dokumenty::KIND,
    &wymiana_dane::KIND,
];

/// A new room on SQLite in memory with every migration of its kinds, for scenarios.
pub fn open_in_memory() -> Result<xmeta_room::MemoryRoom, xmeta_room::RoomError> {
    xmeta_room::MemoryRoom::open(KINDS)
}

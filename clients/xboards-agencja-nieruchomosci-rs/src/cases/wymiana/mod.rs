//! The room of the position `cases/wymiana`: a module per kind, a kind after the kinds it requires.

pub mod wymiana_metryka;
pub mod wymiana_wpisy;
pub mod wymiana_akt;
pub mod wymiana_dokumenty;
pub mod wymiana_dane;

/// The kinds of the room with their migrations, a kind after the kinds it requires.
pub const KINDS: &[(&str, &[&str])] = &[
    (wymiana_metryka::KIND, wymiana_metryka::MIGRATIONS),
    (wymiana_wpisy::KIND, wymiana_wpisy::MIGRATIONS),
    (wymiana_akt::KIND, wymiana_akt::MIGRATIONS),
    (wymiana_dokumenty::KIND, wymiana_dokumenty::MIGRATIONS),
    (wymiana_dane::KIND, wymiana_dane::MIGRATIONS),
];

/// A new room on SQLite in memory with every migration of its kinds, for scenarios.
pub fn open_in_memory() -> Result<xmeta_room::MemoryRoom, xmeta_room::RoomError> {
    let migrations = KINDS.iter().flat_map(|(_, migrations)| migrations.iter().copied());
    xmeta_room::MemoryRoom::open(migrations)
}

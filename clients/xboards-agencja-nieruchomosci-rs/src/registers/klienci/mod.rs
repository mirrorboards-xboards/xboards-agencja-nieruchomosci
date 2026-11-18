//! The room of the position `registers/klienci`: a module per kind, a kind after the kinds it requires.

pub mod klienci_kartoteka;

/// The kinds of the room with their migrations, a kind after the kinds it requires.
pub const KINDS: &[(&str, &[&str])] = &[
    (klienci_kartoteka::KIND, klienci_kartoteka::MIGRATIONS),
];

/// A new room on SQLite in memory with every migration of its kinds, for scenarios.
pub fn open_in_memory() -> Result<xmeta_room::MemoryRoom, xmeta_room::RoomError> {
    let migrations = KINDS.iter().flat_map(|(_, migrations)| migrations.iter().copied());
    xmeta_room::MemoryRoom::open(migrations)
}

//! The room of the position `registers/wykonawcy`: a module per kind, a kind after the kinds it requires.

pub mod wykonawcy_spis;
pub mod wykonawcy_cenniki;

/// The kinds of the room with their migrations, a kind after the kinds it requires.
pub const KINDS: &[(&str, &[&str])] = &[
    (wykonawcy_spis::KIND, wykonawcy_spis::MIGRATIONS),
    (wykonawcy_cenniki::KIND, wykonawcy_cenniki::MIGRATIONS),
];

/// A new room on SQLite in memory with every migration of its kinds, for scenarios.
pub fn open_in_memory() -> Result<xmeta_room::MemoryRoom, xmeta_room::RoomError> {
    let migrations = KINDS.iter().flat_map(|(_, migrations)| migrations.iter().copied());
    xmeta_room::MemoryRoom::open(migrations)
}

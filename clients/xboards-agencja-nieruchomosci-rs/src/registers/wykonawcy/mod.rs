//! The room of the position `registers/wykonawcy`: a module per kind, a kind after the kinds it requires.

pub mod wykonawcy_spis;
pub mod wykonawcy_cenniki;

/// The kinds of the room, a kind after the kinds it requires.
pub const KINDS: &[&xmeta_room::Kind] = &[
    &wykonawcy_spis::KIND,
    &wykonawcy_cenniki::KIND,
];

/// A new room on SQLite in memory with every migration of its kinds, for scenarios.
pub fn open_in_memory() -> Result<xmeta_room::MemoryRoom, xmeta_room::RoomError> {
    xmeta_room::MemoryRoom::open(KINDS)
}

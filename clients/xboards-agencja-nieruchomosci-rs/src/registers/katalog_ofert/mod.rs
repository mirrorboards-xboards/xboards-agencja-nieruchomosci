//! The room of the position `registers/katalog_ofert`: a module per kind, a kind after the kinds it requires.

pub mod katalog_ofert_oferty;

/// The kinds of the room, a kind after the kinds it requires.
pub const KINDS: &[&xmeta_room::Kind] = &[&katalog_ofert_oferty::KIND];

/// A new room on SQLite in memory with every migration of its kinds, for scenarios.
pub fn open_in_memory() -> Result<xmeta_room::MemoryRoom, xmeta_room::RoomError> {
    xmeta_room::MemoryRoom::open(KINDS)
}

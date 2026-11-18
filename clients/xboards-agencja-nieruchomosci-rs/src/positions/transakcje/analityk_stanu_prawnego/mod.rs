//! The room of the position `positions/transakcje/analityk_stanu_prawnego`: a module per kind, a kind after the kinds it requires.

pub mod stanowisko_polecenia;
pub mod stanowisko_zrodla;
pub mod stanowisko_ustalenia;
pub mod analityk_stanu_prawnego_ksiega;
pub mod analityk_stanu_prawnego_sprawdzenia;

/// The kinds of the room, a kind after the kinds it requires.
pub const KINDS: &[&xmeta_room::Kind] = &[
    &stanowisko_polecenia::KIND,
    &stanowisko_zrodla::KIND,
    &stanowisko_ustalenia::KIND,
    &analityk_stanu_prawnego_ksiega::KIND,
    &analityk_stanu_prawnego_sprawdzenia::KIND,
];

/// A new room on SQLite in memory with every migration of its kinds, for scenarios.
pub fn open_in_memory() -> Result<xmeta_room::MemoryRoom, xmeta_room::RoomError> {
    xmeta_room::MemoryRoom::open(KINDS)
}

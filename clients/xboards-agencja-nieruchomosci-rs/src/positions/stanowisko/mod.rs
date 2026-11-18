//! The room of the position `positions/stanowisko`: a module per kind, a kind after the kinds it requires.

pub mod stanowisko_kontakty;
pub mod stanowisko_polecenia;
pub mod stanowisko_zrodla;
pub mod stanowisko_ustalenia;

/// The kinds of the room with their migrations, a kind after the kinds it requires.
pub const KINDS: &[(&str, &[&str])] = &[
    (stanowisko_kontakty::KIND, stanowisko_kontakty::MIGRATIONS),
    (stanowisko_polecenia::KIND, stanowisko_polecenia::MIGRATIONS),
    (stanowisko_zrodla::KIND, stanowisko_zrodla::MIGRATIONS),
    (stanowisko_ustalenia::KIND, stanowisko_ustalenia::MIGRATIONS),
];

/// A new room on SQLite in memory with every migration of its kinds, for scenarios.
pub fn open_in_memory() -> Result<xmeta_room::MemoryRoom, xmeta_room::RoomError> {
    let migrations = KINDS.iter().flat_map(|(_, migrations)| migrations.iter().copied());
    xmeta_room::MemoryRoom::open(migrations)
}

//! The room of the position `positions/kierownik_dzialu`: a module per kind, a kind after the kinds it requires.

pub mod stanowisko_zrodla;
pub mod kierownik_dzialu_agenci;
pub mod kierownik_dzialu_raporty;
pub mod kierownik_dzialu_polisy;
pub mod kierownik_dzialu_spis;
pub mod kierownik_dzialu_standardy;
pub mod kierownik_dzialu_zajrzenia;
pub mod kierownik_dzialu_zwyczaje;

/// The kinds of the room, a kind after the kinds it requires.
pub const KINDS: &[&xmeta_room::Kind] = &[
    &stanowisko_zrodla::KIND,
    &kierownik_dzialu_agenci::KIND,
    &kierownik_dzialu_raporty::KIND,
    &kierownik_dzialu_polisy::KIND,
    &kierownik_dzialu_spis::KIND,
    &kierownik_dzialu_standardy::KIND,
    &kierownik_dzialu_zajrzenia::KIND,
    &kierownik_dzialu_zwyczaje::KIND,
];

/// A new room on SQLite in memory with every migration of its kinds, for scenarios.
pub fn open_in_memory() -> Result<xmeta_room::MemoryRoom, xmeta_room::RoomError> {
    xmeta_room::MemoryRoom::open(KINDS)
}

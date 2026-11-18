//! The room of the position `registers/rejestr_kontroli`: a module per kind, a kind after the kinds it requires.

pub mod rejestr_kontroli_decyzje;
pub mod rejestr_kontroli_klienci;
pub mod rejestr_kontroli_oceny;
pub mod rejestr_kontroli_organizacja;
pub mod rejestr_kontroli_sprawdzenia;
pub mod rejestr_kontroli_watpliwosci;
pub mod rejestr_kontroli_zawiadomienia;

/// The kinds of the room, a kind after the kinds it requires.
pub const KINDS: &[&xmeta_room::Kind] = &[
    &rejestr_kontroli_decyzje::KIND,
    &rejestr_kontroli_klienci::KIND,
    &rejestr_kontroli_oceny::KIND,
    &rejestr_kontroli_organizacja::KIND,
    &rejestr_kontroli_sprawdzenia::KIND,
    &rejestr_kontroli_watpliwosci::KIND,
    &rejestr_kontroli_zawiadomienia::KIND,
];

/// A new room on SQLite in memory with every migration of its kinds, for scenarios.
pub fn open_in_memory() -> Result<xmeta_room::MemoryRoom, xmeta_room::RoomError> {
    xmeta_room::MemoryRoom::open(KINDS)
}

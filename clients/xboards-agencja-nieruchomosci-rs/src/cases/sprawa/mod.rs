//! The room of the position `cases/sprawa`: a module per kind, a kind after the kinds it requires.

pub mod sprawa_decyzje;
pub mod sprawa_nieruchomosci;
pub mod sprawa_uczestnicy;
pub mod sprawa_dokumenty;
pub mod sprawa_umocowania;
pub mod sprawa_korespondencja;
pub mod sprawa_metryka;
pub mod sprawa_notatki;
pub mod sprawa_powiazania;
pub mod sprawa_protokoly;
pub mod sprawa_rachunki;
pub mod sprawa_sprawdzenia;
pub mod sprawa_terminy;
pub mod sprawa_zgody;
pub mod sprawa_umowy;
pub mod sprawa_wyplaty;

/// The kinds of the room, a kind after the kinds it requires.
pub const KINDS: &[&xmeta_room::Kind] = &[
    &sprawa_decyzje::KIND,
    &sprawa_nieruchomosci::KIND,
    &sprawa_uczestnicy::KIND,
    &sprawa_dokumenty::KIND,
    &sprawa_umocowania::KIND,
    &sprawa_korespondencja::KIND,
    &sprawa_metryka::KIND,
    &sprawa_notatki::KIND,
    &sprawa_powiazania::KIND,
    &sprawa_protokoly::KIND,
    &sprawa_rachunki::KIND,
    &sprawa_sprawdzenia::KIND,
    &sprawa_terminy::KIND,
    &sprawa_zgody::KIND,
    &sprawa_umowy::KIND,
    &sprawa_wyplaty::KIND,
];

/// A new room on SQLite in memory with every migration of its kinds, for scenarios.
pub fn open_in_memory() -> Result<xmeta_room::MemoryRoom, xmeta_room::RoomError> {
    xmeta_room::MemoryRoom::open(KINDS)
}

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

/// The kinds of the room with their migrations, a kind after the kinds it requires.
pub const KINDS: &[(&str, &[&str])] = &[
    (sprawa_decyzje::KIND, sprawa_decyzje::MIGRATIONS),
    (sprawa_nieruchomosci::KIND, sprawa_nieruchomosci::MIGRATIONS),
    (sprawa_uczestnicy::KIND, sprawa_uczestnicy::MIGRATIONS),
    (sprawa_dokumenty::KIND, sprawa_dokumenty::MIGRATIONS),
    (sprawa_umocowania::KIND, sprawa_umocowania::MIGRATIONS),
    (sprawa_korespondencja::KIND, sprawa_korespondencja::MIGRATIONS),
    (sprawa_metryka::KIND, sprawa_metryka::MIGRATIONS),
    (sprawa_notatki::KIND, sprawa_notatki::MIGRATIONS),
    (sprawa_powiazania::KIND, sprawa_powiazania::MIGRATIONS),
    (sprawa_protokoly::KIND, sprawa_protokoly::MIGRATIONS),
    (sprawa_rachunki::KIND, sprawa_rachunki::MIGRATIONS),
    (sprawa_sprawdzenia::KIND, sprawa_sprawdzenia::MIGRATIONS),
    (sprawa_terminy::KIND, sprawa_terminy::MIGRATIONS),
    (sprawa_zgody::KIND, sprawa_zgody::MIGRATIONS),
    (sprawa_umowy::KIND, sprawa_umowy::MIGRATIONS),
    (sprawa_wyplaty::KIND, sprawa_wyplaty::MIGRATIONS),
];

/// A new room on SQLite in memory with every migration of its kinds, for scenarios.
pub fn open_in_memory() -> Result<xmeta_room::MemoryRoom, xmeta_room::RoomError> {
    let migrations = KINDS.iter().flat_map(|(_, migrations)| migrations.iter().copied());
    xmeta_room::MemoryRoom::open(migrations)
}

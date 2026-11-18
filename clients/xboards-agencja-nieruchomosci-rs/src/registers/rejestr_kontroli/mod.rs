//! The room of the position `registers/rejestr_kontroli`: a module per kind, a kind after the kinds it requires.

pub mod rejestr_kontroli_decyzje;
pub mod rejestr_kontroli_klienci;
pub mod rejestr_kontroli_oceny;
pub mod rejestr_kontroli_organizacja;
pub mod rejestr_kontroli_sprawdzenia;
pub mod rejestr_kontroli_watpliwosci;
pub mod rejestr_kontroli_zawiadomienia;

/// The kinds of the room with their migrations, a kind after the kinds it requires.
pub const KINDS: &[(&str, &[&str])] = &[
    (rejestr_kontroli_decyzje::KIND, rejestr_kontroli_decyzje::MIGRATIONS),
    (rejestr_kontroli_klienci::KIND, rejestr_kontroli_klienci::MIGRATIONS),
    (rejestr_kontroli_oceny::KIND, rejestr_kontroli_oceny::MIGRATIONS),
    (rejestr_kontroli_organizacja::KIND, rejestr_kontroli_organizacja::MIGRATIONS),
    (rejestr_kontroli_sprawdzenia::KIND, rejestr_kontroli_sprawdzenia::MIGRATIONS),
    (rejestr_kontroli_watpliwosci::KIND, rejestr_kontroli_watpliwosci::MIGRATIONS),
    (rejestr_kontroli_zawiadomienia::KIND, rejestr_kontroli_zawiadomienia::MIGRATIONS),
];

/// A new room on SQLite in memory with every migration of its kinds, for scenarios.
pub fn open_in_memory() -> Result<xmeta_room::MemoryRoom, xmeta_room::RoomError> {
    let migrations = KINDS.iter().flat_map(|(_, migrations)| migrations.iter().copied());
    xmeta_room::MemoryRoom::open(migrations)
}

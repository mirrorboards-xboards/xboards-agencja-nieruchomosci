//! The room of the position `positions/kontrola/kierownik_kontroli`: a module per kind, a kind after the kinds it requires.

pub mod stanowisko_polecenia;
pub mod stanowisko_zrodla;
pub mod stanowisko_ustalenia;
pub mod kierownik_dzialu_agenci;
pub mod kierownik_dzialu_raporty;
pub mod kierownik_dzialu_standardy;
pub mod kierownik_dzialu_zajrzenia;
pub mod kierownik_dzialu_zwyczaje;
pub mod kierownik_kontroli_dane_osobowe;
pub mod kierownik_kontroli_oceny;
pub mod kierownik_kontroli_projekty;

/// The kinds of the room with their migrations, a kind after the kinds it requires.
pub const KINDS: &[(&str, &[&str])] = &[
    (stanowisko_polecenia::KIND, stanowisko_polecenia::MIGRATIONS),
    (stanowisko_zrodla::KIND, stanowisko_zrodla::MIGRATIONS),
    (stanowisko_ustalenia::KIND, stanowisko_ustalenia::MIGRATIONS),
    (kierownik_dzialu_agenci::KIND, kierownik_dzialu_agenci::MIGRATIONS),
    (kierownik_dzialu_raporty::KIND, kierownik_dzialu_raporty::MIGRATIONS),
    (kierownik_dzialu_standardy::KIND, kierownik_dzialu_standardy::MIGRATIONS),
    (kierownik_dzialu_zajrzenia::KIND, kierownik_dzialu_zajrzenia::MIGRATIONS),
    (kierownik_dzialu_zwyczaje::KIND, kierownik_dzialu_zwyczaje::MIGRATIONS),
    (kierownik_kontroli_dane_osobowe::KIND, kierownik_kontroli_dane_osobowe::MIGRATIONS),
    (kierownik_kontroli_oceny::KIND, kierownik_kontroli_oceny::MIGRATIONS),
    (kierownik_kontroli_projekty::KIND, kierownik_kontroli_projekty::MIGRATIONS),
];

/// A new room on SQLite in memory with every migration of its kinds, for scenarios.
pub fn open_in_memory() -> Result<xmeta_room::MemoryRoom, xmeta_room::RoomError> {
    let migrations = KINDS.iter().flat_map(|(_, migrations)| migrations.iter().copied());
    xmeta_room::MemoryRoom::open(migrations)
}

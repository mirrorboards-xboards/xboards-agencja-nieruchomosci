//! The room of the position `positions/biuro/kierownik_biura`: a module per kind, a kind after the kinds it requires.

pub mod stanowisko_polecenia;
pub mod stanowisko_zrodla;
pub mod stanowisko_ustalenia;
pub mod kierownik_biura_decyzje;
pub mod kierownik_biura_meldunki;
pub mod kierownik_biura_nieobecnosci;
pub mod kierownik_biura_polisy;
pub mod kierownik_biura_przydzial;
pub mod kierownik_biura_sprawozdania;

/// The kinds of the room, a kind after the kinds it requires.
pub const KINDS: &[&xmeta_room::Kind] = &[
    &stanowisko_polecenia::KIND,
    &stanowisko_zrodla::KIND,
    &stanowisko_ustalenia::KIND,
    &kierownik_biura_decyzje::KIND,
    &kierownik_biura_meldunki::KIND,
    &kierownik_biura_nieobecnosci::KIND,
    &kierownik_biura_polisy::KIND,
    &kierownik_biura_przydzial::KIND,
    &kierownik_biura_sprawozdania::KIND,
];

/// A new room on SQLite in memory with every migration of its kinds, for scenarios.
pub fn open_in_memory() -> Result<xmeta_room::MemoryRoom, xmeta_room::RoomError> {
    xmeta_room::MemoryRoom::open(KINDS)
}

//! The kind `konsjerz-reguly` of the room `positions/najem/konsjerz`: its migrations, the values of its enumerated
//! columns and its operations.

/// The kind's name.
pub const KIND: &str = "konsjerz-reguly";

/// The migrations of the kind, in the order of their numbers.
pub const MIGRATIONS: &[&str] = &[
    r#"-- Reguły Konsjerża na tabelach pamięci wspólnej. Rodzaj nie ma własnych tabel.
--
-- Konsjerż ocenia rzetelność rodzaju zgłoszenia, np. „brak ciepłej wody”, a nie konkretnego gościa
-- (D14). Lokal ma wielu gości, więc źródło o rodzaju osoba zrobiłoby z pamięci kartotekę gości.
-- Rodzaju źródła się nie zmienia (stanowisko_zrodla_tozsamosc_zrodla_niezmienna), więc wystarcza
-- reguła przy wpisie.
CREATE TRIGGER konsjerz_reguly_zrodlo_nie_osoba
BEFORE INSERT ON stanowisko_zrodla_wpisy
WHEN NEW.rodzaj = 'osoba'
BEGIN
  SELECT RAISE(ABORT, 'Konsjerż ocenia rodzaj zgłoszenia, a nie gościa; źródło nie jest osobą');
END;
"#,
];

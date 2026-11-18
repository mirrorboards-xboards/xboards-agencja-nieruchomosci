//! The kind `poszukiwanie-reguly` of the room `cases/poszukiwanie`: its migrations, the values of its enumerated
//! columns and its operations.

/// The kind: its name, the kinds it requires and its migrations in the order of their
/// numbers.
pub const KIND: xmeta_room::Kind = xmeta_room::Kind {
    name: "poszukiwanie-reguly",
    requires: &[
        &super::sprawa_metryka::KIND,
        &super::sprawa_rachunki::KIND,
        &super::sprawa_uczestnicy::KIND,
        &super::sprawa_umowy::KIND,
        &super::sprawa_wyplaty::KIND,
        &super::sprawa_zgody::KIND,
    ],
    migrations: &[
        r#"-- Reguły teczki poszukiwania na tabelach rdzenia: typ pokoju, etapy z przejściami, role, rodzaje
-- zgód, rodzaj umowy z biurem i zakaz pieniędzy. Rodzaj nie ma własnych tabel; wskazuje tylko
-- rdzeń, więc jest pierwszym rodzajem typu.
--
-- Reguły na rdzeniu, które zależą od tabel typu, leżą w migracjach tych rodzajów: wejście
-- w propozycje wymaga potwierdzonego profilu (poszukiwanie-profile). Decyzje własne typu (analiza
-- poza standardem, szerszy zakres danych klienta dla drugiej strony) są decyzjami „inna”, które
-- wskazują wiersz typu; ich reguły leżą w poszukiwanie-kredyt i poszukiwanie-propozycje.
--
-- Przerwana sprawa się zamyka albo wraca do etapu sprzed przerwania za decyzją wznowienie_sprawy
-- (rdzeń, sprawa_metryka_po_przerwaniu_zamkniecie_albo_powrot, T1).

-- Teczka poszukiwania ma typ pokoju poszukiwanie.
CREATE TRIGGER poszukiwanie_reguly_typ_pokoju
BEFORE INSERT ON sprawa_metryka_wpis
WHEN NEW.typ_pokoju <> 'poszukiwanie'
BEGIN
  SELECT RAISE(ABORT, 'teczka poszukiwania ma typ pokoju poszukiwanie');
END;

-- Etapy poszukiwania z rozdziału „Etapy” profilu: przyjeta, weryfikacja_klienta,
-- umowa_posrednictwa, profil_poszukiwania, propozycje, negocjacje (po wysłaniu oferty klienta)
-- i oferta_przyjeta. Z negocjacji sprawa wraca do propozycji, a z przyjętej oferty też, gdy
-- oferta nie doszła do skutku. Od weryfikacji do przyjętej oferty sprawa przechodzi do
-- koniec_poszukiwania, a z niego do zamknieta. Do przerwana przechodzi się z każdego etapu przed
-- zamknięciem, a z przerwana do zamknieta albo z powrotem do etapu sprzed przerwania
-- (sprawa_metryka_powrot_z_przerwania). Pierwszy etap (przyjeta) i decyzje przerwania i wznowienia
-- pilnuje rdzeń.
CREATE TRIGGER poszukiwanie_reguly_przejscia_etapow
BEFORE INSERT ON sprawa_metryka_etapy
WHEN EXISTS (
    SELECT 1
    FROM sprawa_metryka_etapy
  )
  AND NOT coalesce(
    (
      SELECT
        (poprzedni.etap = 'przyjeta' AND NEW.etap = 'weryfikacja_klienta')
        OR (poprzedni.etap = 'weryfikacja_klienta' AND NEW.etap = 'umowa_posrednictwa')
        OR (poprzedni.etap = 'umowa_posrednictwa' AND NEW.etap = 'profil_poszukiwania')
        OR (poprzedni.etap = 'profil_poszukiwania' AND NEW.etap = 'propozycje')
        OR (poprzedni.etap = 'propozycje' AND NEW.etap = 'negocjacje')
        OR (poprzedni.etap = 'negocjacje' AND NEW.etap IN ('propozycje', 'oferta_przyjeta'))
        OR (poprzedni.etap = 'oferta_przyjeta' AND NEW.etap = 'propozycje')
        OR (
          poprzedni.etap IN ('weryfikacja_klienta', 'umowa_posrednictwa', 'profil_poszukiwania', 'propozycje', 'negocjacje', 'oferta_przyjeta')
          AND NEW.etap = 'koniec_poszukiwania'
        )
        OR (poprzedni.etap = 'koniec_poszukiwania' AND NEW.etap = 'zamknieta')
        OR (poprzedni.etap NOT IN ('przerwana', 'zamknieta') AND NEW.etap = 'przerwana')
        OR (
          poprzedni.etap = 'przerwana'
          AND (
            NEW.etap = 'zamknieta'
            OR NEW.etap IN (
              SELECT etap
              FROM sprawa_metryka_powrot_z_przerwania
            )
          )
        )
      FROM sprawa_metryka_etapy AS poprzedni
      ORDER BY poprzedni.id DESC
      LIMIT 1
    ),
    0
  )
BEGIN
  SELECT RAISE(ABORT, 'etap poszukiwania przechodzi: przyjeta, weryfikacja_klienta, umowa_posrednictwa, profil_poszukiwania, propozycje i negocjacje, oferta_przyjeta, potem koniec_poszukiwania i zamknieta; przerwana z każdego etapu, a po niej zamknieta albo powrót do etapu sprzed przerwania');
END;

-- Profil poszukiwania ustala się według podpisanej umowy pośrednictwa (art. 180 ust. 3 ugn).
CREATE TRIGGER poszukiwanie_reguly_profil_po_zawarciu_umowy
BEFORE INSERT ON sprawa_metryka_etapy
WHEN NEW.etap = 'profil_poszukiwania'
  AND NOT EXISTS (
    SELECT 1
    FROM sprawa_umowy_wpisy
    WHERE rodzaj = 'posrednictwo'
      AND stan = 'zawarta'
  )
BEGIN
  SELECT RAISE(ABORT, 'etap profil_poszukiwania wymaga zawartej umowy pośrednictwa');
END;

-- Role w teczce poszukiwania: klient (strona umowy), współnabywca, który nie jest stroną umowy,
-- np. partner, i małżonek. Pełnomocnik wynika z umocowań. Strona, która sprzedaje albo wynajmuje,
-- jest w teczce tylko tekstem przy propozycji i ofercie (R1).
CREATE TRIGGER poszukiwanie_reguly_role
BEFORE INSERT ON sprawa_uczestnicy_role
WHEN NEW.rola NOT IN ('klient', 'wspolnabywca', 'malzonek')
BEGIN
  SELECT RAISE(ABORT, 'rola w teczce poszukiwania to klient, wspolnabywca albo malzonek');
END;

-- Zgody klienta z rozdziału „Sprawy za zgodą i zakazane” i zgody rdzenia.
CREATE TRIGGER poszukiwanie_reguly_rodzaje_zgod
BEFORE INSERT ON sprawa_zgody_wpisy
WHEN NEW.rodzaj NOT IN (
  'zadanie_rozpoczecia_pracy',
  'odstapienie',
  'potwierdzenie_profilu',
  'potwierdzenie_oferty'
)
BEGIN
  SELECT RAISE(ABORT, 'zgoda w teczce poszukiwania to potwierdzenie_profilu, potwierdzenie_oferty albo zgoda rdzenia');
END;

-- Teczka poszukiwania ma umowę pośrednictwa.
CREATE TRIGGER poszukiwanie_reguly_umowa_posrednictwa
BEFORE INSERT ON sprawa_umowy_wpisy
WHEN NEW.rodzaj <> 'posrednictwo'
BEGIN
  SELECT RAISE(ABORT, 'umowa w teczce poszukiwania to umowa pośrednictwa');
END;

-- Teczka poszukiwania nie przyjmuje ani nie przekazuje pieniędzy: opłata rezerwacyjna, zadatek
-- i kaucja należą do teczki transakcji albo najmu (R4).
CREATE TRIGGER poszukiwanie_reguly_bez_rachunkow
BEFORE INSERT ON sprawa_rachunki_wpisy
BEGIN
  SELECT RAISE(ABORT, 'teczka poszukiwania nie przyjmuje pieniędzy: nie ma rachunków ani wypłat');
END;

CREATE TRIGGER poszukiwanie_reguly_bez_wyplat
BEFORE INSERT ON sprawa_wyplaty_wpisy
BEGIN
  SELECT RAISE(ABORT, 'teczka poszukiwania nie przyjmuje pieniędzy: nie ma rachunków ani wypłat');
END;
"#,
    ],
};

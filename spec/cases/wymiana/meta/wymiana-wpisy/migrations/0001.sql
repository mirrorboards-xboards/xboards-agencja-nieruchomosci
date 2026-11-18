-- Dziennik wpisów pokoju wymiany: każdy wpis obu stron, także wiadomości, z autorem, stroną
-- i czasem. Pola typowe wpisu leżą w tabelach szczegółów innych rodzajów, które go wskazują
-- (wymiana-dokumenty, wymiana-dane, wymiana-akt; decyzja T11). Podpis autora daje serwer pokoju.
--
-- Wpisu się nie zmienia ani nie usuwa. Poprawka jest nowym wpisem, który wskazuje poprzedni.
-- Teczka transakcji w biurze i teczka notariusza wskazują wpis jego identyfikatorem, np. wpływ
-- z kanału pokoj_wymiany, więc wpis ma UUID.
--
-- Do pokoju nie trafia nic z przeciwdziałania praniu pieniędzy ani dane jednej strony, których
-- akt nie wymaga. Nie ma na to kolumn, a treść wiadomości pilnuje aplikacja.
CREATE TABLE wymiana_wpisy_wpisy (
  id TEXT PRIMARY KEY,
  -- Strona pokoju, w której imieniu pisze autor; deklarację porównuje aplikacja (decyzja T10).
  strona TEXT NOT NULL CHECK (strona IN ('biuro', 'kancelaria')),
  -- Tożsamość członka pokoju, który podpisał wpis.
  autor TEXT NOT NULL CHECK (autor <> ''),
  rodzaj TEXT NOT NULL CHECK (
    rodzaj IN (
      'lista_dokumentow',
      'dokument',
      'pelnomocnictwo',
      'zgoda_stron',
      'projekt_aktu',
      'koszty',
      'uwagi_do_projektu',
      'dane_osoby',
      'dane_nieruchomosci',
      'rachunek',
      'kredyt',
      'propozycja_terminu',
      'stan_terminu',
      'pierwokup',
      'po_akcie',
      'wiadomosc'
    )
  ),
  chwila TEXT NOT NULL,
  -- Wpis, na który ten odpowiada, np. odpowiedź na pytanie o ważność zaświadczenia.
  odpowiada_na_id TEXT REFERENCES wymiana_wpisy_wpisy (id),
  -- Wpis, który ten poprawia; poprawiony wpis zostaje w pokoju.
  poprawia_id TEXT REFERENCES wymiana_wpisy_wpisy (id),
  tresc TEXT CHECK (tresc <> ''),
  -- Plik w repozytorium plików pokoju i skrót jego treści, który obejmuje podpis wpisu.
  sciezka TEXT CHECK (sciezka <> ''),
  skrot TEXT CHECK (skrot <> ''),

  CONSTRAINT plik_ze_skrotem CHECK ((sciezka IS NULL) = (skrot IS NULL)),
  CONSTRAINT dokument_z_plikiem CHECK (
    rodzaj NOT IN ('dokument', 'pelnomocnictwo', 'zgoda_stron', 'projekt_aktu') OR sciezka IS NOT NULL
  ),
  CONSTRAINT wiadomosc_z_trescia CHECK (rodzaj <> 'wiadomosc' OR tresc IS NOT NULL),
  -- Klucz obcy przepuściłby wpis, który wskazuje sam siebie; poprawkę pilnuje trigger niżej.
  CONSTRAINT odpowiedz_nie_wskazuje_siebie CHECK (odpowiada_na_id IS NOT id),
  -- Listę dokumentów, projekt aktu, koszty, pierwokup i informację po akcie zna i wybiera
  -- kancelaria (art. 18 § 1 PoN).
  CONSTRAINT wpis_tylko_od_kancelarii CHECK (
    rodzaj NOT IN ('lista_dokumentow', 'projekt_aktu', 'koszty', 'pierwokup', 'po_akcie')
    OR strona = 'kancelaria'
  ),
  -- Dokumenty, pełnomocnictwa, zgody, uwagi i dane stron biuro przekazuje w imieniu stron.
  CONSTRAINT wpis_tylko_od_biura CHECK (
    rodzaj NOT IN (
      'dokument',
      'pelnomocnictwo',
      'zgoda_stron',
      'uwagi_do_projektu',
      'dane_osoby',
      'dane_nieruchomosci',
      'kredyt'
    )
    OR strona = 'biuro'
  )
) STRICT;

CREATE TRIGGER wymiana_wpisy_wymagaja_metryki
BEFORE INSERT ON wymiana_wpisy_wpisy
WHEN NOT EXISTS (
  SELECT 1
  FROM wymiana_metryka_wpis
)
BEGIN
  SELECT RAISE(ABORT, 'wpis powstaje dopiero w pokoju z metryką');
END;

-- Po zamknięciu nikt w pokoju nie pisze, a członkowie mogą go tylko czytać.
CREATE TRIGGER wymiana_wpisy_zamkniety_pokoj_bez_wpisow
BEFORE INSERT ON wymiana_wpisy_wpisy
WHEN (
  SELECT stan
  FROM wymiana_metryka_stan
) = 'zamkniety'
BEGIN
  SELECT RAISE(ABORT, 'w zamkniętym pokoju nie powstaje nowy wpis');
END;

-- Poprawka zastępuje dane tego samego rodzaju i tej samej strony: kancelaria nie poprawia
-- pełnomocnictwa biura, a nowe dane osoby nie poprawiają rachunku.
CREATE TRIGGER wymiana_wpisy_poprawka_tego_samego_rodzaju
BEFORE INSERT ON wymiana_wpisy_wpisy
WHEN NEW.poprawia_id IS NOT NULL
  AND NOT EXISTS (
    SELECT 1
    FROM wymiana_wpisy_wpisy AS poprawiany
    WHERE poprawiany.id = NEW.poprawia_id
      AND poprawiany.rodzaj = NEW.rodzaj
      AND poprawiany.strona = NEW.strona
  )
BEGIN
  SELECT RAISE(ABORT, 'poprawka wskazuje wpis tego samego rodzaju i tej samej strony');
END;

CREATE TRIGGER wymiana_wpisy_wpisy_bez_zmian
BEFORE UPDATE ON wymiana_wpisy_wpisy
BEGIN
  SELECT RAISE(ABORT, 'wiersza pokoju wymiany się nie zmienia; poprawka to nowy wpis');
END;

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers.
CREATE TRIGGER wymiana_wpisy_wpisy_bez_zastepowania
BEFORE INSERT ON wymiana_wpisy_wpisy
WHEN EXISTS (
  SELECT 1
  FROM wymiana_wpisy_wpisy
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza pokoju wymiany nie zastępuje się nowym wpisem');
END;

CREATE TRIGGER wymiana_wpisy_wpisy_bez_usuwania
BEFORE DELETE ON wymiana_wpisy_wpisy
BEGIN
  SELECT RAISE(ABORT, 'wierszy pokoju wymiany się nie usuwa (R9)');
END;

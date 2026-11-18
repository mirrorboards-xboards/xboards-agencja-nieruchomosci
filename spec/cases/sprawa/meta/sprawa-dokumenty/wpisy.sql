-- name: dokumenty :many
-- Lista dokumentów teczki w kolejności wpisania.
SELECT *
FROM sprawa_dokumenty_wpisy
ORDER BY rowid;

-- name: dokumenty_do_zebrania :many
-- Dokumenty, które są wymagane albo zamówione i jeszcze nie przyszły.
SELECT *
FROM sprawa_dokumenty_wpisy
WHERE stan IN ('wymagany', 'zamowiony')
ORDER BY rowid;

-- name: wpisz_wymagany_dokument :exec
-- Dokument, którego wymaga przepis, notariusz, bank albo umowa; czeka na zamówienie.
INSERT INTO sprawa_dokumenty_wpisy (id, rodzaj, nazwa, dotyczy_uczestnik_id,
  dotyczy_nieruchomosc_id, wymaga, stan)
VALUES (:id, :rodzaj, :nazwa, :dotyczy_uczestnik_id, :dotyczy_nieruchomosc_id, :wymaga, 'wymagany');

-- name: odnotuj_zamowienie_dokumentu :exec
-- Biuro zamówiło dokument, np. wysyłką do spółdzielni w imieniu klienta.
UPDATE sprawa_dokumenty_wpisy
SET stan = 'zamowiony'
WHERE id = :id;

-- name: wpisz_otrzymany_dokument :exec
-- Dokument, który przyszedł bez wcześniejszego wymagania, od razu z plikiem.
-- param sciezka: string
-- param skrot: string
INSERT INTO sprawa_dokumenty_wpisy (id, rodzaj, nazwa, dotyczy_uczestnik_id,
  dotyczy_nieruchomosc_id, wystawca, data_wystawienia, waznosc_do, waznosc_zrodlo, wymaga,
  pochodzenie, kanal, postac, sciezka, skrot, stan)
VALUES (:id, :rodzaj, :nazwa, :dotyczy_uczestnik_id, :dotyczy_nieruchomosc_id, :wystawca,
  :data_wystawienia, :waznosc_do, :waznosc_zrodlo, :wymaga, :pochodzenie, :kanal, :postac, :sciezka,
  :skrot, 'otrzymany');

-- name: odnotuj_otrzymanie_dokumentu :exec
-- Wymagany albo zamówiony dokument przyszedł: wystawca, ważność, pochodzenie i plik.
-- param sciezka: string
-- param skrot: string
UPDATE sprawa_dokumenty_wpisy
SET stan = 'otrzymany',
  wystawca = :wystawca,
  data_wystawienia = :data_wystawienia,
  waznosc_do = :waznosc_do,
  waznosc_zrodlo = :waznosc_zrodlo,
  pochodzenie = :pochodzenie,
  kanal = :kanal,
  postac = :postac,
  sciezka = :sciezka,
  skrot = :skrot
WHERE id = :id;

-- name: oznacz_dokument_jako_sprawdzony :exec
-- Agent sprawy porównał dokument z wymaganiem: jest kompletny i czytelny.
UPDATE sprawa_dokumenty_wpisy
SET stan = 'sprawdzony'
WHERE id = :id;

-- name: odrzuc_dokument :exec
-- Dokument nie nadaje się, np. „nieczytelna pieczęć”; agent zamawia nowy jako nowy wiersz.
-- param powod_odrzucenia: string
UPDATE sprawa_dokumenty_wpisy
SET stan = 'odrzucony',
  powod_odrzucenia = :powod_odrzucenia
WHERE id = :id;

-- name: uznaj_dokument_za_zbedny :exec
-- Wymagany albo zamówiony dokument przestał być potrzebny.
UPDATE sprawa_dokumenty_wpisy
SET stan = 'zbedny'
WHERE id = :id;

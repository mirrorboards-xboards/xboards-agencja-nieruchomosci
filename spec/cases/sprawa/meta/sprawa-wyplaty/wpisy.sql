-- name: wyplaty :many
-- Wypłaty pieniędzy klienta, najnowsze pierwsze.
SELECT *
FROM sprawa_wyplaty_wpisy
ORDER BY utworzono DESC, id;

-- name: wyplaty_przygotowane :many
-- Wypłaty, które czekają na sprawdzenie, zgodę albo wykonanie.
SELECT *
FROM sprawa_wyplaty_wpisy
WHERE stan = 'przygotowana'
ORDER BY utworzono, id;

-- name: przygotuj_wyplate :exec
-- Wypłata z podstawą z typu sprawy, kwotą w groszach, wyliczeniem i dopuszczonym rachunkiem
-- odbiorcy; identyfikator (UUID) nadaje aplikacja, bo wskazują go decyzja i rejestr kontroli.
INSERT INTO sprawa_wyplaty_wpisy (id, podstawa_tabela, podstawa_id, kwota, wyliczenie_sciezka,
  wyliczenie_skrot, odbiorca_uczestnik_id, rachunek_id, utworzono, stan)
VALUES (:id, :podstawa_tabela, :podstawa_id, :kwota, :wyliczenie_sciezka, :wyliczenie_skrot,
  :odbiorca_uczestnik_id, :rachunek_id, :utworzono, 'przygotowana');

-- name: wskaz_sprawdzenie_wyplaty :exec
-- Sprawdzenie rachunku, które Kontroler zrobił po przygotowaniu wypłaty.
-- param sprawdzenie_id: string
UPDATE sprawa_wyplaty_wpisy
SET sprawdzenie_id = :sprawdzenie_id
WHERE id = :id;

-- name: wskaz_zgode_na_wyplate :exec
-- Zatwierdzona decyzja wyplata, która wskazuje tę wypłatę.
-- param decyzja_id: string
UPDATE sprawa_wyplaty_wpisy
SET decyzja_id = :decyzja_id
WHERE id = :id;

-- name: odnotuj_wykonanie_wyplaty :exec
-- Człowiek, który zlecił przelew, wpisuje wykonanie z potwierdzeniem z banku.
-- param wykonal: string
-- param wykonano: string
-- param potwierdzenie_dokument_id: string
UPDATE sprawa_wyplaty_wpisy
SET stan = 'wykonana',
  wykonal = :wykonal,
  wykonano = :wykonano,
  potwierdzenie_dokument_id = :potwierdzenie_dokument_id
WHERE id = :id;

-- name: anuluj_wyplate :exec
-- Wypłata, która nie zostanie wykonana, np. po zmianie rozliczenia; nowa to nowy wiersz.
UPDATE sprawa_wyplaty_wpisy
SET stan = 'anulowana'
WHERE id = :id;

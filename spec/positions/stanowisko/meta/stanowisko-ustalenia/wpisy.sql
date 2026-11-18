-- name: ustalenia :many
-- Aktualne ustalenia agenta, najnowsze pierwsze.
SELECT *
FROM stanowisko_ustalenia_wpisy
WHERE nieaktualne_od IS NULL
ORDER BY ustalono DESC, id;

-- name: ustalenia_od_zrodla :many
-- Aktualne ustalenia, które stoją na jednym źródle; po nieprawdzie tego źródła agent ocenia je na
-- nowo i oznacza jako niepewne to, czego nie potwierdził niezależnie.
SELECT *
FROM stanowisko_ustalenia_wpisy
WHERE zrodlo_id = :zrodlo_id
  AND nieaktualne_od IS NULL
ORDER BY ustalono, id;

-- name: ustalenia_z_wiersza :many
-- Aktualne ustalenia, które wynikają z jednego wiersza teczki; po zmianie dokumentu stają się
-- nieaktualne.
SELECT *
FROM stanowisko_ustalenia_wpisy
WHERE wynika_z_odnosnik = :wynika_z_odnosnik
  AND nieaktualne_od IS NULL
ORDER BY ustalono, id;

-- name: ustalenia_o :many
-- Aktualne ustalenia o jednej rzeczy, np. o lokalu, propozycji albo pobycie.
SELECT *
FROM stanowisko_ustalenia_wpisy
WHERE dotyczy_odnosnik = :dotyczy_odnosnik
  AND nieaktualne_od IS NULL
ORDER BY ustalono, id;

-- name: zapisz_ustalenie :exec
-- Nowe ustalenie ze źródłem albo wierszem teczki; identyfikator (UUID) nadaje aplikacja, żeby
-- nowsze ustalenie mogło go od razu wskazać.
INSERT INTO stanowisko_ustalenia_wpisy (id, tresc, zrodlo_id, wynika_z_tabela, wynika_z_odnosnik,
  dotyczy_tabela, dotyczy_odnosnik, pewnosc, sposob_sprawdzenia, ustalono)
VALUES (:id, :tresc, :zrodlo_id, :wynika_z_tabela, :wynika_z_odnosnik, :dotyczy_tabela,
  :dotyczy_odnosnik, :pewnosc, :sposob_sprawdzenia, :ustalono);

-- name: ocen_ustalenie_na_nowo :exec
-- Nowa pewność ustalenia, np. po nieprawdzie źródła; potwierdzenie wymaga sposobu sprawdzenia.
UPDATE stanowisko_ustalenia_wpisy
SET pewnosc = :pewnosc,
  sposob_sprawdzenia = :sposob_sprawdzenia
WHERE id = :id;

-- name: odnotuj_wykorzystanie_ustalenia :exec
-- Ustalenie weszło do wiersza teczki albo raportu.
-- param wykorzystano_tabela: string
-- param wykorzystano_odnosnik: string
UPDATE stanowisko_ustalenia_wpisy
SET wykorzystano_tabela = :wykorzystano_tabela,
  wykorzystano_odnosnik = :wykorzystano_odnosnik
WHERE id = :id;

-- name: uniewaznij_ustalenie :exec
-- Ustalenie przestało być aktualne, np. bo dokument się zmienił; nowsze ustalenie, jeśli jest,
-- wskazuje się jako zastępujące.
-- param nieaktualne_od: string
UPDATE stanowisko_ustalenia_wpisy
SET nieaktualne_od = :nieaktualne_od,
  zastapione_przez_id = :zastapione_przez_id
WHERE id = :id;

-- name: sprawdzenia :many
-- Aktualne sprawdzenia (niezastąpione), od najnowszego.
SELECT *
FROM kontroler_sprawdzenia_wpisy
WHERE zastapione_przez_id IS NULL
ORDER BY sprawdzono DESC, id;

-- name: sprawdzenia_teczki :many
-- Aktualne sprawdzenia jednej teczki klienta, od najstarszego.
SELECT *
FROM kontroler_sprawdzenia_wpisy
WHERE teczka_odnosnik = :teczka_odnosnik
  AND zastapione_przez_id IS NULL
ORDER BY sprawdzono, id;

-- name: sprawdzenia_do_przekazania :many
-- Aktualne sprawdzenia robocze, które czekają na przekazanie Kierownikowi kontroli.
SELECT *
FROM kontroler_sprawdzenia_wpisy
WHERE przekazano IS NULL
  AND zastapione_przez_id IS NULL
ORDER BY sprawdzono, id;

-- name: zapisz_sprawdzenie :exec
-- Sprawdzenie z wynikiem; identyfikator (UUID) nadaje aplikacja, żeby poprzednie sprawdzenie mogło
-- go wskazać jako zastępujące.
INSERT INTO kontroler_sprawdzenia_wpisy (id, rodzaj, teczka_odnosnik, dotyczy_tabela,
  dotyczy_odnosnik, zrodlo, zrodlo_data, zrodlo_id, wynik, opis, proponowany_poziom, czynniki,
  powod_watpliwosci, sprawdzono)
VALUES (:id, :rodzaj, :teczka_odnosnik, :dotyczy_tabela, :dotyczy_odnosnik, :zrodlo, :zrodlo_data,
  :zrodlo_id, :wynik, :opis, :proponowany_poziom, :czynniki, :powod_watpliwosci, :sprawdzono);

-- name: odnotuj_przekazanie_sprawdzenia :exec
-- Sprawdzenie przekazane Kierownikowi kontroli; od tej chwili się nie zmienia.
-- param przekazano: string
UPDATE kontroler_sprawdzenia_wpisy
SET przekazano = :przekazano
WHERE id = :id;

-- name: odnotuj_wpis_sprawdzenia_w_rejestrze :exec
-- Kierownik kontroli podał wpis w rejestrze kontroli, który powstał z przekazanego sprawdzenia;
-- opis, czynniki i proponowany poziom leżą odtąd tylko w rejestrze (D13).
-- param rejestr_odnosnik: string
UPDATE kontroler_sprawdzenia_wpisy
SET rejestr_odnosnik = :rejestr_odnosnik,
  opis = NULL,
  czynniki = NULL,
  proponowany_poziom = NULL
WHERE id = :id;

-- name: zastap_sprawdzenie :exec
-- Poprzednie sprawdzenie staje się nieaktualne, bo zastępuje je ponowne, zapisane wcześniej.
-- param zastapione_przez_id: string
UPDATE kontroler_sprawdzenia_wpisy
SET zastapione_przez_id = :zastapione_przez_id
WHERE id = :id;

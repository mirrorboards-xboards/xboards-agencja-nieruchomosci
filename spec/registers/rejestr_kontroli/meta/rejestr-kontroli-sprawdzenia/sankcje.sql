-- name: sprawdzenia_sankcji_klienta :many
-- Sprawdzenia list sankcyjnych klienta, najnowsze pierwsze.
SELECT *
FROM rejestr_kontroli_sprawdzenia_sankcje
WHERE klient_id = :klient_id
ORDER BY id DESC;

-- name: wpisz_sprawdzenie_sankcji :exec
-- Sprawdzenie list sankcyjnych z wynikiem; zamrożenie wpisuje wpisz_zamrozenie.
INSERT INTO rejestr_kontroli_sprawdzenia_sankcje (klient_id, listy, wynik, zrodlo, raport_odnosnik,
  sprawdzil, sprawdzono, wpisal, wpisano, aktualne, poprzedni_id)
VALUES (:klient_id, :listy, :wynik, :zrodlo, :raport_odnosnik, :sprawdzil, :sprawdzono, :wpisal,
  :wpisano, 1, :poprzedni_id);

-- name: wpisz_zamrozenie :exec
-- Chwila zamrożenia wartości majątkowych klienta po trafieniu na liście; wpisuje się raz.
-- param zamrozono: string
UPDATE rejestr_kontroli_sprawdzenia_sankcje
SET zamrozono = :zamrozono
WHERE id = :id;

-- name: wpisz_informacje_o_zamrozeniu :exec
-- Chwila przekazania informacji o zamrożeniu Generalnemu Inspektorowi; wpisuje się raz.
-- param zamrozenie_informacja: string
UPDATE rejestr_kontroli_sprawdzenia_sankcje
SET zamrozenie_informacja = :zamrozenie_informacja
WHERE id = :id;

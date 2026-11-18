-- name: zadania_organow :many
-- Żądania organów, najnowsze pierwsze.
SELECT *
FROM rejestr_kontroli_zawiadomienia_zadania
ORDER BY zadano DESC, id;

-- name: wpisz_zadanie_organu :exec
-- Żądanie organu, które odebrał właściciel Kierownika kontroli.
INSERT INTO rejestr_kontroli_zawiadomienia_zadania (organ, zadano, tresc, odebral, wpisal, wpisano)
VALUES (:organ, :zadano, :tresc, :odebral, :wpisal, :wpisano);

-- name: wpisz_odpowiedz_na_zadanie :exec
-- Kiedy i co biuro przekazało organowi; wpisuje się raz.
-- param przekazano: string
-- param co_przekazano: string
UPDATE rejestr_kontroli_zawiadomienia_zadania
SET przekazano = :przekazano,
  co_przekazano = :co_przekazano
WHERE id = :id;

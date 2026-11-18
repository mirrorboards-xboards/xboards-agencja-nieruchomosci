-- name: oceny_ryzyka_biura :many
-- Wersje oceny ryzyka biura, najnowsza pierwsza.
SELECT *
FROM rejestr_kontroli_oceny_biura
ORDER BY wersja DESC;

-- name: wpisz_ocene_ryzyka_biura :exec
-- Nowa wersja oceny ryzyka biura po akceptacji właściciela biura. Identyfikator (UUID) nadaje
-- aplikacja, zanim powstanie prośba o akceptację.
INSERT INTO rejestr_kontroli_oceny_biura (id, wersja, czynniki, sporzadzono, aktualizacja_do,
  dokument_sciezka, dokument_skrot, akceptacja_decyzja_id, wpisal, wpisano)
VALUES (:id, :wersja, :czynniki, :sporzadzono, :aktualizacja_do, :dokument_sciezka, :dokument_skrot,
  :akceptacja_decyzja_id, :wpisal, :wpisano);

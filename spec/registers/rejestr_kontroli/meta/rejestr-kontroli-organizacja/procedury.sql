-- name: procedury :many
-- Wersje procedury wewnętrznej, najnowsza pierwsza.
SELECT *
FROM rejestr_kontroli_organizacja_procedury
ORDER BY wersja DESC;

-- name: wpisz_procedure :exec
-- Nowa wersja procedury po akceptacji właściciela biura. Identyfikator (UUID) nadaje aplikacja,
-- zanim powstanie prośba o akceptację.
INSERT INTO rejestr_kontroli_organizacja_procedury (id, wersja, dokument_sciezka, dokument_skrot,
  akceptacja_decyzja_id, wprowadzono, wpisal, wpisano)
VALUES (:id, :wersja, :dokument_sciezka, :dokument_skrot, :akceptacja_decyzja_id, :wprowadzono,
  :wpisal, :wpisano);

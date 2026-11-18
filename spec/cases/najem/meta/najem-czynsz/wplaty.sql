-- name: wplaty_czynszu :many
-- Wpłaty w kolejności chwili wpływu.
SELECT *
FROM najem_czynsz_wplaty
ORDER BY chwila, id;

-- name: wpisz_wplate_czynszu :exec
-- Wpłata z płacącym, rachunkiem i potwierdzeniem; identyfikator (UUID) nadaje aplikacja, bo
-- wskazuje go przekazanie wynajmującemu.
INSERT INTO najem_czynsz_wplaty (id, najem_id, chwila, kwota, placacy_uczestnik_id, na_rachunek,
  rachunek_id, potwierdzenie, dokument_id)
VALUES (:id, :najem_id, :chwila, :kwota, :placacy_uczestnik_id, :na_rachunek, :rachunek_id,
  :potwierdzenie, :dokument_id);

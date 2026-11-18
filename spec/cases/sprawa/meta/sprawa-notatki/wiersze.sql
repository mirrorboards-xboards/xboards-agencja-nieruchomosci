-- name: wiersze_notatek :many
-- Wiersze teczki, których dotyczą notatki.
SELECT *
FROM sprawa_notatki_wiersze
ORDER BY notatka_id, id;

-- name: wskaz_wiersz_notatki :exec
-- Wiersz teczki, którego dotyczy notatka.
INSERT INTO sprawa_notatki_wiersze (notatka_id, dotyczy_tabela, dotyczy_id)
VALUES (:notatka_id, :dotyczy_tabela, :dotyczy_id);

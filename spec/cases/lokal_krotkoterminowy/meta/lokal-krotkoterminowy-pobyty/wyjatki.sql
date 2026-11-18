-- name: wyjatki_pobytow :many
-- Wyjątki od regulaminu w kolejności wpisania.
SELECT *
FROM lokal_krotkoterminowy_pobyty_wyjatki
ORDER BY rowid;

-- name: wpisz_wyjatek_pobytu :exec
-- Wyjątek od regulaminu za zgodą człowieka. Identyfikator (UUID) nadaje aplikacja, bo wskazuje go
-- decyzja.
INSERT INTO lokal_krotkoterminowy_pobyty_wyjatki (id, pobyt_id, opis, decyzja_id)
VALUES (:id, :pobyt_id, :opis, :decyzja_id);

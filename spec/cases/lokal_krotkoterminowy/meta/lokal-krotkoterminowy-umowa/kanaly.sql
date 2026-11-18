-- name: kanaly_warunkow :many
-- Kanały sprzedaży każdej wersji warunków.
SELECT *
FROM lokal_krotkoterminowy_umowa_kanaly
ORDER BY warunki_id, id;

-- name: dopisz_kanal_warunkow :exec
-- Kanał sprzedaży z kontem biura albo właściciela w projekcie warunków.
INSERT INTO lokal_krotkoterminowy_umowa_kanaly (warunki_id, kanal, konto)
VALUES (:warunki_id, :kanal, :konto);

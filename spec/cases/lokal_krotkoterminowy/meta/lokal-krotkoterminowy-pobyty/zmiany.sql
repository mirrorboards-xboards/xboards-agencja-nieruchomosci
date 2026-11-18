-- name: zmiany_pobytow :many
-- Zmiany pobytów w kolejności czasu.
SELECT *
FROM lokal_krotkoterminowy_pobyty_zmiany
ORDER BY chwila, id;

-- name: wpisz_zmiane_pobytu :exec
-- Zmiana terminu, liczby gości albo ceny, albo odwołanie, z przyczyną i decyzją Zarządcy.
INSERT INTO lokal_krotkoterminowy_pobyty_zmiany (pobyt_id, chwila, co, przyczyna, zdecydowal)
VALUES (:pobyt_id, :chwila, :co, :przyczyna, :zdecydowal);

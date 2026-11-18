-- name: powiazania :many
-- Powiązania teczki w kolejności wpisania.
SELECT *
FROM sprawa_powiazania_wpisy
ORDER BY id;

-- name: wpisz_powiazanie :exec
-- Odnośnik do innej teczki albo pokoju, z rodzajem związku.
INSERT INTO sprawa_powiazania_wpisy (rodzaj, typ_teczki, oznaczenie, odnosnik, wpisal, wpisano)
VALUES (:rodzaj, :typ_teczki, :oznaczenie, :odnosnik, :wpisal, :wpisano);

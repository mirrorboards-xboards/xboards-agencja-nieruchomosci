-- name: zamieszkujacy :many
-- Osoby zamieszkujące z najemcą z datą zamieszkania i wyprowadzki.
SELECT *
FROM najem_umowy_zamieszkujacy
ORDER BY od, id;

-- name: wpisz_zamieszkujacego :exec
-- Osoba, która zamieszkała z najemcą, i od kiedy.
INSERT INTO najem_umowy_zamieszkujacy (najem_id, uczestnik_id, od)
VALUES (:najem_id, :uczestnik_id, :od);

-- name: odnotuj_wyprowadzke_zamieszkujacego :exec
-- Osoba przestała mieszkać z najemcą.
-- param do: string
UPDATE najem_umowy_zamieszkujacy
SET do = :do
WHERE id = :id;

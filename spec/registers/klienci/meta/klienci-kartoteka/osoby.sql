-- name: osoby_klienta :many
-- Osoby działające za podmiot i osoby kontaktowe, bieżące i zakończone.
SELECT *
FROM klienci_kartoteka_osoby
WHERE klient_id = :klient_id
ORDER BY id DESC;

-- name: wpisz_osobe_klienta :exec
-- Osoba, która podpisała umowę za podmiot (z podstawą umocowania), albo osoba kontaktowa.
INSERT INTO klienci_kartoteka_osoby (klient_id, funkcja, imiona, nazwisko, kontakt,
  podstawa_odnosnik, od, zrodlo_odnosnik)
VALUES (:klient_id, :funkcja, :imiona, :nazwisko, :kontakt, :podstawa_odnosnik, :od,
  :zrodlo_odnosnik);

-- name: zakoncz_osobe_klienta :exec
-- Osoba, która przestała działać za podmiot albo rozmawiać z biurem.
UPDATE klienci_kartoteka_osoby
SET do = :do
WHERE id = :id;

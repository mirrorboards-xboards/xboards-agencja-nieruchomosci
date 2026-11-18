-- name: adresy_uczestnikow :many
-- Adresy wszystkich uczestników teczki.
SELECT *
FROM sprawa_uczestnicy_adresy
ORDER BY uczestnik_id, rodzaj;

-- name: wpisz_adres_uczestnika :exec
-- Adres uczestnika; każdy rodzaj adresu raz na uczestnika.
INSERT INTO sprawa_uczestnicy_adresy (uczestnik_id, rodzaj, kraj, miejscowosc, ulica, numer_budynku,
  numer_lokalu, kod_pocztowy)
VALUES (:uczestnik_id, :rodzaj, :kraj, :miejscowosc, :ulica, :numer_budynku, :numer_lokalu,
  :kod_pocztowy);

-- name: zmien_adres_uczestnika :exec
-- Nowy adres tego samego rodzaju, np. po przeprowadzce.
UPDATE sprawa_uczestnicy_adresy
SET kraj = :kraj,
  miejscowosc = :miejscowosc,
  ulica = :ulica,
  numer_budynku = :numer_budynku,
  numer_lokalu = :numer_lokalu,
  kod_pocztowy = :kod_pocztowy
WHERE id = :id;

-- name: wykresl_adres_uczestnika :exec
-- Poprawka pomyłki: adres wpisany niepotrzebnie albo nie tej osobie (D14).
DELETE FROM sprawa_uczestnicy_adresy
WHERE id = :id;

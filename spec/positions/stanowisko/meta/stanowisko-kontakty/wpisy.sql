-- name: kontakty :many
-- Aktualne kontakty agenta według teczki i roli.
SELECT *
FROM stanowisko_kontakty_wpisy
WHERE nieaktualny_od IS NULL
ORDER BY dotyczy_odnosnik, rola, id;

-- name: kontakty_sprawy :many
-- Aktualne kontakty jednej teczki albo jednego lokalu.
SELECT *
FROM stanowisko_kontakty_wpisy
WHERE dotyczy_odnosnik = :dotyczy_odnosnik
  AND nieaktualny_od IS NULL
ORDER BY rola, id;

-- name: zapisz_kontakt :exec
-- Nowy kontakt; uczestnika teczki albo wpis rejestru wskazuje odnośnik zamiast nazwy i adresu.
INSERT INTO stanowisko_kontakty_wpisy (dotyczy_odnosnik, instytucja, rola, wpis_odnosnik, nazwa, adres,
  kanal, pory, zapisano)
VALUES (:dotyczy_odnosnik, :instytucja, :rola, :wpis_odnosnik, :nazwa, :adres, :kanal, :pory,
  :zapisano);

-- name: uniewaznij_kontakt :exec
-- Kontakt przestał być aktualny, np. bo zmienił się administrator albo numer; nowy kontakt to nowy
-- wiersz.
-- param nieaktualny_od: string
UPDATE stanowisko_kontakty_wpisy
SET nieaktualny_od = :nieaktualny_od
WHERE id = :id;

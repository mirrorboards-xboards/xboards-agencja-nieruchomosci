-- name: znaki_sprawy :many
-- Znaki jednej sprawy, także wycofane, od najstarszego.
SELECT *
FROM kierownik_dzialu_spis_znaki
WHERE sprawa_id = :sprawa_id
ORDER BY dodano, id;

-- name: znaki_do_przekazania_recepcji :many
-- Znaki, których Recepcja jeszcze nie dostała, i wycofania znaków, które już dostała.
SELECT *
FROM kierownik_dzialu_spis_znaki
WHERE (przekazano IS NULL AND wycofano IS NULL)
  OR (przekazano IS NOT NULL AND wycofano IS NOT NULL AND wycofanie_przekazano IS NULL)
ORDER BY sprawa_id, id;

-- name: dodaj_znak :exec
-- Nowy znak rozpoznawczy sprawy, np. adres nieruchomości.
INSERT INTO kierownik_dzialu_spis_znaki (sprawa_id, rodzaj, wartosc, dodano)
VALUES (:sprawa_id, :rodzaj, :wartosc, :dodano);

-- name: odnotuj_przekazanie_znaku :exec
-- Recepcja dostała znak.
-- param przekazano: string
UPDATE kierownik_dzialu_spis_znaki
SET przekazano = :przekazano
WHERE id = :id;

-- name: wycofaj_znak :exec
-- Znak przestał wskazywać sprawę, np. klient zmienił adres poczty; zostaje w spisie jako wycofany.
-- param wycofano: string
UPDATE kierownik_dzialu_spis_znaki
SET wycofano = :wycofano
WHERE id = :id;

-- name: odnotuj_przekazanie_wycofania :exec
-- Recepcja dostała wycofanie znaku.
-- param wycofanie_przekazano: string
UPDATE kierownik_dzialu_spis_znaki
SET wycofanie_przekazano = :wycofanie_przekazano
WHERE id = :id;

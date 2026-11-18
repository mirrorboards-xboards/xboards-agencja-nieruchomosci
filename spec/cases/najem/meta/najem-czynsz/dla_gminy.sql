-- name: dane_dla_gminy :many
-- Dane o czynszu dla gminy w kolejności okresów.
SELECT *
FROM najem_czynsz_dla_gminy
ORDER BY okres, id;

-- name: przygotuj_dane_dla_gminy :exec
-- Dane tego najmu za okres, z umowy i z dokumentów budynku.
INSERT INTO najem_czynsz_dla_gminy (najem_id, okres, czynsz, gmina, rok_budowy, stan_techniczny,
  powierzchnia, standard)
VALUES (:najem_id, :okres, :czynsz, :gmina, :rok_budowy, :stan_techniczny, :powierzchnia,
  :standard);

-- name: popraw_dane_dla_gminy :exec
-- Poprawka danych, dopóki Opiekun ich nie oddał.
UPDATE najem_czynsz_dla_gminy
SET czynsz = :czynsz,
  rok_budowy = :rok_budowy,
  stan_techniczny = :stan_techniczny,
  powierzchnia = :powierzchnia,
  standard = :standard
WHERE id = :id;

-- name: odnotuj_oddanie_danych_dla_gminy :exec
-- Opiekun oddał dane Kierownikowi najmu.
-- param oddano: string
UPDATE najem_czynsz_dla_gminy
SET oddano = :oddano
WHERE id = :id;

-- name: odnotuj_wyslanie_danych_dla_gminy :exec
-- Człowiek wysłał zestawienie gminie.
-- param wyslano: string
-- param wyslal: string
UPDATE najem_czynsz_dla_gminy
SET wyslano = :wyslano,
  wyslal = :wyslal
WHERE id = :id;

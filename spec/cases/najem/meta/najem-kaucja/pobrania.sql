-- name: pobrania_kaucji :many
-- Pobrane kaucje z tym, gdzie leżą.
SELECT *
FROM najem_kaucja_pobrania
ORDER BY wplacono, id;

-- name: wpisz_pobranie_kaucji :exec
-- Kaucja wpłacona przy zawarciu umowy; identyfikator (UUID) nadaje aplikacja.
INSERT INTO najem_kaucja_pobrania (id, najem_id, kwota, krotnosc, czynsz_przy_zawarciu, wplacono,
  lezy, rachunek_id)
VALUES (:id, :najem_id, :kwota, :krotnosc, :czynsz_przy_zawarciu, :wplacono, :lezy, :rachunek_id);

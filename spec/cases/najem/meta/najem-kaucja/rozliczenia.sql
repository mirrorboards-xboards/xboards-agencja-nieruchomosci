-- name: rozliczenia_kaucji :many
-- Rozliczenia kaucji z kwotą zwaloryzowaną i stanem.
SELECT *
FROM najem_kaucja_rozliczenia
ORDER BY oproznienie, id;

-- name: przygotuj_rozliczenie_kaucji :exec
-- Projekt rozliczenia z wyliczeniem programu po opróżnieniu lokalu; identyfikator (UUID) nadaje
-- aplikacja, bo wskazują go potrącenia i wypłata zwrotu.
INSERT INTO najem_kaucja_rozliczenia (id, pobranie_id, oproznienie, czynsz_w_dniu_zwrotu,
  kwota_zwaloryzowana, wyliczenie_sciezka, wyliczenie_skrot, stan)
VALUES (:id, :pobranie_id, :oproznienie, :czynsz_w_dniu_zwrotu, :kwota_zwaloryzowana,
  :wyliczenie_sciezka, :wyliczenie_skrot, 'projekt');

-- name: zatwierdz_rozliczenie_kaucji :exec
-- Każde potrącenie ma zgodę człowieka; rozliczenie może iść do stron.
UPDATE najem_kaucja_rozliczenia
SET stan = 'zatwierdzone'
WHERE id = :id;

-- name: odnotuj_wyslanie_rozliczenia_kaucji :exec
-- Rozliczenie wysłane stronom według wzoru.
-- param wysylka_id: string
UPDATE najem_kaucja_rozliczenia
SET stan = 'wyslane',
  wysylka_id = :wysylka_id
WHERE id = :id;

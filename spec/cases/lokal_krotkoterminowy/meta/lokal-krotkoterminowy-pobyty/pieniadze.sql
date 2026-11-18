-- name: pieniadze_pobytow :many
-- Zwroty, rekompensaty, roszczenia, wnioski kanału i odzyskane kwoty w kolejności czasu.
SELECT *
FROM lokal_krotkoterminowy_pobyty_pieniadze
ORDER BY chwila, id;

-- name: wpisz_pieniadze_pobytu :exec
-- Kwota poza rezerwacją; bez zgody człowieka tylko zwrot według zasad kanału i kwota odzyskana.
-- Identyfikator (UUID) nadaje aplikacja, bo wskazują go decyzja i pozycja zestawienia.
INSERT INTO lokal_krotkoterminowy_pobyty_pieniadze (id, pobyt_id, rodzaj, wedlug_zasad_kanalu, kwota,
  chwila, opis, decyzja_id)
VALUES (:id, :pobyt_id, :rodzaj, :wedlug_zasad_kanalu, :kwota, :chwila, :opis, :decyzja_id);

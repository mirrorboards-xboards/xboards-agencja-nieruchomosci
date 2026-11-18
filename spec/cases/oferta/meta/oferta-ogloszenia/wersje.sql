-- name: wersje_ogloszen :many
-- Wersje treści ogłoszeń od najstarszej.
SELECT *
FROM oferta_ogloszenia_wersje
ORDER BY ogloszenie_id, numer;

-- name: najnowsze_wersje_ogloszen :many
-- Najnowsza wersja treści każdego ogłoszenia.
SELECT *
FROM oferta_ogloszenia_wersje_najnowsze
ORDER BY ogloszenie_id;

-- name: dopisz_wersje_ogloszenia :exec
-- Nowa wersja treści z kolejnym numerem; wersja opublikowanego ogłoszenia przechodzi reguły
-- publikacji.
INSERT INTO oferta_ogloszenia_wersje (ogloszenie_id, numer, od, tytul, cena, lokalizacja,
  wskazniki_energetyczne, tresc_sciezka, tresc_skrot)
VALUES (:ogloszenie_id, :numer, :od, :tytul, :cena, :lokalizacja, :wskazniki_energetyczne,
  :tresc_sciezka, :tresc_skrot);

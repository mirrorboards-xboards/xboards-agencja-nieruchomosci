-- name: propozycje_cen :many
-- Propozycje ceny i czynszu biura, najnowsze pierwsze.
SELECT *
FROM oferta_ceny_propozycje
ORDER BY utworzono DESC, id;

-- name: porownania_transakcji :many
-- Transakcje porównane w propozycjach ceny, w kolejności wpisania.
SELECT *
FROM oferta_ceny_porownania
ORDER BY propozycja_id, id;

-- name: wpisz_propozycje_ceny :exec
-- Propozycja ceny albo czynszu z uzasadnieniem i plikiem wyliczenia z programu; identyfikator
-- (UUID) nadaje aplikacja, bo wskazują go porównania.
INSERT INTO oferta_ceny_propozycje (id, dotyczy, kwota, uzasadnienie, utworzono, wyliczenie_sciezka,
  wyliczenie_skrot)
VALUES (:id, :dotyczy, :kwota, :uzasadnienie, :utworzono, :wyliczenie_sciezka, :wyliczenie_skrot);

-- name: dopisz_porownanie_transakcji :exec
-- Transakcja z rejestru cen nieruchomości porównana w propozycji, z różnicami wobec oferty.
INSERT INTO oferta_ceny_porownania (propozycja_id, data_transakcji, cena, powierzchnia, roznice,
  zrodlo)
VALUES (:propozycja_id, :data_transakcji, :cena, :powierzchnia, :roznice, :zrodlo);

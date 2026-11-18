-- name: statystyki_ogloszen :many
-- Odczyty statystyk portali, najnowsze pierwsze.
SELECT *
FROM oferta_ogloszenia_statystyki
ORDER BY odczytano DESC, id;

-- name: wpisz_statystyki_ogloszenia :exec
-- Liczby wyświetleń i zapytań podane przez portal, z chwilą odczytu.
INSERT INTO oferta_ogloszenia_statystyki (ogloszenie_id, odczytano, wyswietlenia, zapytania)
VALUES (:ogloszenie_id, :odczytano, :wyswietlenia, :zapytania);

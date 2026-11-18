-- name: ryzyka_transakcji :many
-- Ryzyka wszystkich analiz w kolejności wpisania.
SELECT *
FROM transakcja_analizy_ryzyka
ORDER BY analiza_id, id;

-- name: dopisz_ryzyko_transakcji :exec
-- Ryzyko z aktualnej analizy, ze skutkiem dla stron.
INSERT INTO transakcja_analizy_ryzyka (analiza_id, opis, skutek)
VALUES (:analiza_id, :opis, :skutek);

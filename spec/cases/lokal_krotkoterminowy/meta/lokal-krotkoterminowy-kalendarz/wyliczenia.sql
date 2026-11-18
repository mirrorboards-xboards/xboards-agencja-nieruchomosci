-- name: wyliczenia_cen :many
-- Wyliczenia cen w kolejności czasu.
SELECT *
FROM lokal_krotkoterminowy_kalendarz_wyliczenia
ORDER BY chwila, id;

-- name: wpisz_wyliczenie_cen :exec
-- Dane wejściowe wyliczenia, ich źródła z datą i plik wyniku programu.
INSERT INTO lokal_krotkoterminowy_kalendarz_wyliczenia (id, chwila, dane_wejsciowe, zrodla, sciezka,
  skrot)
VALUES (:id, :chwila, :dane_wejsciowe, :zrodla, :sciezka, :skrot);

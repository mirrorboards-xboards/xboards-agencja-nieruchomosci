-- name: przeglady :many
-- Przeglądy lokalu w kolejności dat.
SELECT *
FROM najem_usterki_przeglady
ORDER BY data, id;

-- name: wpisz_przeglad :exec
-- Okresowy przegląd z wynikiem i protokołem, jeśli jest.
INSERT INTO najem_usterki_przeglady (data, wynik, dokument_id)
VALUES (:data, :wynik, :dokument_id);

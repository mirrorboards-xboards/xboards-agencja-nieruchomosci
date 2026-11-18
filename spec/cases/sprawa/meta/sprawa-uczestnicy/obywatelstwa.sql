-- name: obywatelstwa_uczestnikow :many
-- Obywatelstwa wszystkich uczestników teczki.
SELECT *
FROM sprawa_uczestnicy_obywatelstwa
ORDER BY uczestnik_id, panstwo;

-- name: wpisz_obywatelstwo_uczestnika :exec
-- Obywatelstwo osoby fizycznej, gdy nabywca może być cudzoziemcem.
INSERT INTO sprawa_uczestnicy_obywatelstwa (uczestnik_id, panstwo)
VALUES (:uczestnik_id, :panstwo);

-- name: wykresl_obywatelstwo_uczestnika :exec
-- Poprawka pomyłki w obywatelstwie (D14).
DELETE FROM sprawa_uczestnicy_obywatelstwa
WHERE id = :id;

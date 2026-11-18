-- name: role_uczestnikow :many
-- Role uczestników w teczce, w kolejności nadania.
SELECT *
FROM sprawa_uczestnicy_role
ORDER BY id;

-- name: nadaj_role_uczestnikowi :exec
-- Rola uczestnika w teczce, z listy ról typu sprawy; małżonek wskazuje osobę, której jest
-- małżonkiem.
INSERT INTO sprawa_uczestnicy_role (uczestnik_id, rola, dotyczy_uczestnik_id)
VALUES (:uczestnik_id, :rola, :dotyczy_uczestnik_id);

-- name: zakoncz_role_uczestnika :exec
-- Koniec roli, która przestała obowiązywać albo była pomyłką; rola zostaje w teczce z datą
-- i powodem (R9).
UPDATE sprawa_uczestnicy_role
SET zakonczono = :zakonczono, powod_zakonczenia = :powod_zakonczenia
WHERE id = :id;

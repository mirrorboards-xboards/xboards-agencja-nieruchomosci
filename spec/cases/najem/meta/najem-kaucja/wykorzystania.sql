-- name: wykorzystania_kaucji :many
-- Pokrycia zaległości z kaucji przy najmie instytucjonalnym.
SELECT *
FROM najem_kaucja_wykorzystania
ORDER BY chwila, id;

-- name: wpisz_wykorzystanie_kaucji :exec
-- Właściciel pokrył zaległość z kaucji; termin uzupełnienia z umowy albo z pisma.
INSERT INTO najem_kaucja_wykorzystania (pobranie_id, kwota, chwila, zaleglosc_id, uzupelnic_do)
VALUES (:pobranie_id, :kwota, :chwila, :zaleglosc_id, :uzupelnic_do);

-- name: odnotuj_uzupelnienie_kaucji :exec
-- Najemca uzupełnił kaucję.
-- param uzupelniono: string
UPDATE najem_kaucja_wykorzystania
SET uzupelniono = :uzupelniono
WHERE id = :id;

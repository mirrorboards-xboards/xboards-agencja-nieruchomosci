-- name: polecenia :many
-- Wszystkie polecenia agenta, najnowsze pierwsze.
SELECT *
FROM stanowisko_polecenia_wpisy
ORDER BY otrzymano DESC, id;

-- name: polecenia_otwarte :many
-- Polecenia przyjęte, ale jeszcze niewykonane, od najstarszego.
SELECT *
FROM stanowisko_polecenia_wpisy
WHERE stan = 'przyjete'
ORDER BY otrzymano, id;

-- name: przyjmij_polecenie :exec
-- Podpisane polecenie właściciela albo upoważnionego przez niego człowieka; podpis sprawdziła
-- aplikacja.
INSERT INTO stanowisko_polecenia_wpisy (wydal, wydal_jako, otrzymano, tresc, podpis, stan)
VALUES (:wydal, :wydal_jako, :otrzymano, :tresc, :podpis, 'przyjete');

-- name: odnotuj_wykonanie_polecenia :exec
-- Polecenie wykonane.
-- param zamknieto: string
UPDATE stanowisko_polecenia_wpisy
SET stan = 'wykonane',
  zamknieto = :zamknieto,
  uwagi = :uwagi
WHERE id = :id;

-- name: odmow_polecenia :exec
-- Agent odmawia polecenia, np. sprzecznego z normą stanowiska, i pisze dlaczego.
-- param zamknieto: string
-- param uwagi: string
UPDATE stanowisko_polecenia_wpisy
SET stan = 'odmowione',
  zamknieto = :zamknieto,
  uwagi = :uwagi
WHERE id = :id;

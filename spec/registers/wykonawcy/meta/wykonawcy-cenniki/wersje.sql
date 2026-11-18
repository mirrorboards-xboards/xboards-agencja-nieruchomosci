-- name: wersje_cennika :many
-- Wersje jednego cennika, najnowsza pierwsza.
SELECT *
FROM wykonawcy_cenniki_wersje
WHERE cennik_id = :cennik_id
ORDER BY numer DESC;

-- name: obowiazujaca_wersja_cennika :one
-- Zamknięta wersja cennika, która obowiązuje w danym dniu; null przed pierwszą.
-- param dzien: string
SELECT *
FROM wykonawcy_cenniki_wersje
WHERE cennik_id = :cennik_id
  AND stan = 'zamknieta'
  AND obowiazuje_od <= :dzien
ORDER BY numer DESC
LIMIT 1;

-- name: wpisz_wersje_cennika :exec
-- Projekt nowej wersji po decyzji o cenniku; pozycje dopisuje dopisz_pozycje_cennika, a wersję
-- zamyka zamknij_wersje_cennika. Identyfikator (UUID) nadaje aplikacja, bo wskazują go pozycje
-- i zlecenia w teczkach lokali.
INSERT INTO wykonawcy_cenniki_wersje (id, cennik_id, numer, obowiazuje_od, wpisano, stan, decyzja_id)
VALUES (:id, :cennik_id, :numer, :obowiazuje_od, :wpisano, 'projekt', :decyzja_id);

-- name: zamknij_wersje_cennika :exec
-- Wersja ze wszystkimi pozycjami, których skrót objął podpis decyzji; od teraz się nie zmienia.
UPDATE wykonawcy_cenniki_wersje
SET stan = 'zamknieta'
WHERE id = :id;

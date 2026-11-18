-- name: warunki_zarzadzania :many
-- Wersje warunków umowy o zarządzanie; bieżąca jest ostatnia.
SELECT *
FROM najem_umowy_zarzadzanie
ORDER BY id;

-- name: wpisz_warunki_zarzadzania :exec
-- Kwota napraw i przepływ pieniędzy: pierwsza wersja od zawarcia umowy o zarządzanie, kolejna od
-- aneksu tej umowy.
INSERT INTO najem_umowy_zarzadzanie (umowa_id, kwota_napraw, przeplyw, aneks_zdarzenie_id, od)
VALUES (:umowa_id, :kwota_napraw, :przeplyw, :aneks_zdarzenie_id, :od);

-- name: zrodla_srodkow_klienta :many
-- Pochodzenie pieniędzy klienta z dokumentami, najnowsze pierwsze.
SELECT *
FROM rejestr_kontroli_sprawdzenia_zrodla_srodkow
WHERE klient_id = :klient_id
ORDER BY id DESC;

-- name: wpisz_zrodlo_srodkow :exec
-- Pochodzenie pieniędzy klienta z dokumentem.
INSERT INTO rejestr_kontroli_sprawdzenia_zrodla_srodkow (klient_id, pochodzenie_srodkow,
  dokument_sciezka, dokument_skrot, zrodlo, raport_odnosnik, sprawdzil, sprawdzono, wpisal, wpisano,
  aktualne, poprzedni_id)
VALUES (:klient_id, :pochodzenie_srodkow, :dokument_sciezka, :dokument_skrot, :zrodlo,
  :raport_odnosnik, :sprawdzil, :sprawdzono, :wpisal, :wpisano, 1, :poprzedni_id);

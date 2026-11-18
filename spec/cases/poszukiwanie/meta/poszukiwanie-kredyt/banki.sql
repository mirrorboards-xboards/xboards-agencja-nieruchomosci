-- name: banki_klienta :many
-- Banki, które wybrał klient, z jego wiadomością i wymaganiami banku.
SELECT *
FROM poszukiwanie_kredyt_banki
ORDER BY wybrano, id;

-- name: wpisz_bank_klienta :exec
-- Bank wybrany przez klienta z wpływem, z którego to wiadomo; biuro banków nie poleca (R2).
INSERT INTO poszukiwanie_kredyt_banki (bank, wybrano, zrodlo_wplyw_id)
VALUES (:bank, :wybrano, :zrodlo_wplyw_id);

-- name: odnotuj_wymagania_banku :exec
-- Skąd teczka zna wymagania banku wobec dokumentów i z kiedy; wpisuje się je raz.
-- param wymagania_zrodlo: string
-- param wymagania_data: string
UPDATE poszukiwanie_kredyt_banki
SET wymagania_zrodlo = :wymagania_zrodlo,
  wymagania_data = :wymagania_data
WHERE id = :id;

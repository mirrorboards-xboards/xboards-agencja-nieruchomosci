-- name: ceny_ofertowe :many
-- Historia cen ofertowych od najstarszej: każda zmiana wskazuje poprzednią cenę.
SELECT *
FROM oferta_ceny_ofertowe
ORDER BY dotyczy, obowiazuje_od, id;

-- name: ceny_ofertowe_biezace :many
-- Ceny ofertowe, które obowiązują: sprzedaży, najmu albo obie.
SELECT *
FROM oferta_ceny_biezace
ORDER BY dotyczy;

-- name: wpisz_cene_ofertowa :exec
-- Cena ofertowa zatwierdzona przez klienta. Zmiana ceny wskazuje cenę, która obowiązywała,
-- i ma powód; identyfikator (UUID) nadaje aplikacja, bo wskazuje go zgoda klienta.
INSERT INTO oferta_ceny_ofertowe (id, dotyczy, kwota, zgoda_id, obowiazuje_od, poprzednia_id, powod)
VALUES (:id, :dotyczy, :kwota, :zgoda_id, :obowiazuje_od, :poprzednia_id, :powod);

-- name: odnotuj_cene_w_ogloszeniach :exec
-- Dzień, w którym cena weszła do ogłoszeń; wpisuje się go raz.
-- param w_ogloszeniach_od: string
UPDATE oferta_ceny_ofertowe
SET w_ogloszeniach_od = :w_ogloszeniach_od
WHERE id = :id;

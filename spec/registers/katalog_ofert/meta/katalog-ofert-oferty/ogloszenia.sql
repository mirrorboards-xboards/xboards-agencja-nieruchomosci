-- name: ogloszenia_ofert :many
-- Ogłoszenia wszystkich ofert, trwające i zakończone.
SELECT *
FROM katalog_ofert_oferty_ogloszenia
ORDER BY oferta_id, id;

-- name: oferta_po_ogloszeniu :one
-- Ogłoszenie o numerze z wiadomości portalu; wskazuje ofertę.
SELECT *
FROM katalog_ofert_oferty_ogloszenia
WHERE portal = :portal
  AND numer_ogloszenia = :numer_ogloszenia;

-- name: wpisz_ogloszenie :exec
-- Ogłoszenie oferty publicznej z numerem nadanym przez portal.
INSERT INTO katalog_ofert_oferty_ogloszenia (oferta_id, portal, numer_ogloszenia, od)
VALUES (:oferta_id, :portal, :numer_ogloszenia, :od);

-- name: zakoncz_ogloszenie :exec
-- Ogłoszenie zdjęte z portalu.
UPDATE katalog_ofert_oferty_ogloszenia
SET do = :do
WHERE id = :id;

-- name: statusy_oferty :many
-- Historia statusu oferty ze źródłami.
SELECT *
FROM katalog_ofert_oferty_statusy
WHERE oferta_id = :oferta_id
ORDER BY id;

-- name: zmien_status_oferty :exec
-- Rezerwacja, powrót do aktywnych, sprzedaż albo najem, wycofanie; ze źródłem w teczce oferty.
INSERT INTO katalog_ofert_oferty_statusy (oferta_id, status, od, zrodlo_odnosnik, wpisal)
VALUES (:oferta_id, :status, :od, :zrodlo_odnosnik, :wpisal);

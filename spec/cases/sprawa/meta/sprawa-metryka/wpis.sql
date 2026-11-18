-- name: metryka :one
-- Metryka teczki: oznaczenie, typ pokoju, kto przyjął sprawę i kiedy.
SELECT *
FROM sprawa_metryka_wpis
WHERE id = 1;

-- name: zaloz_metryke :exec
-- Założenie teczki przez człowieka, który przyjmuje sprawę; sprawa wchodzi w etap „przyjęta”.
INSERT INTO sprawa_metryka_wpis (id, oznaczenie, typ_pokoju, przyjal, przyjeto)
VALUES (1, :oznaczenie, :typ_pokoju, :przyjal, :przyjeto);

INSERT INTO sprawa_metryka_etapy (etap, od, zmienil)
VALUES ('przyjeta', :przyjeto, :przyjal);

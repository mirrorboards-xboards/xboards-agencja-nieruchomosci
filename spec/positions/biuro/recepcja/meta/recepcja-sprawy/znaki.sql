-- name: znaki_sprawy :many
-- Znaki rozpoznawcze jednej sprawy, także wycofane.
SELECT *
FROM recepcja_sprawy_znaki
WHERE sprawa_id = :sprawa_id
ORDER BY dodano, id;

-- name: sprawy_po_znaku :many
-- Sprawy, do których pasuje znak z wiadomości, razem ze znakiem. Wiadomość przypisuje się tylko
-- wtedy, gdy pasuje dokładnie jedna sprawa; gdy kilka albo żadna, Recepcja zapisuje kartę.
SELECT znak.id AS znak_id,
  sprawa.*
FROM recepcja_sprawy_znaki AS znak
JOIN recepcja_sprawy_wpisy AS sprawa ON sprawa.id = znak.sprawa_id
WHERE znak.rodzaj = :rodzaj
  AND znak.wartosc = :wartosc
  AND znak.wycofano IS NULL
ORDER BY sprawa.oznaczenie, sprawa.id;

-- name: dodaj_znak_sprawy :exec
-- Znak rozpoznawczy przekazany przez kierownika działu.
INSERT INTO recepcja_sprawy_znaki (sprawa_id, rodzaj, wartosc, przekazal, dodano)
VALUES (:sprawa_id, :rodzaj, :wartosc, :przekazal, :dodano);

-- name: wycofaj_znak_sprawy :exec
-- Wycofanie znaku przekazane przez kierownika działu; znak zostaje jako wycofany.
-- param wycofano: string
UPDATE recepcja_sprawy_znaki
SET wycofano = :wycofano
WHERE id = :id;

-- name: zrodla :many
-- Źródła wszystkich danych teczki.
SELECT *
FROM sprawa_dokumenty_zrodla
ORDER BY dotyczy_tabela, dotyczy_id, pole, id;

-- name: zrodla_wiersza :many
-- Źródła pól jednego wiersza teczki, najnowsze pierwsze.
SELECT *
FROM sprawa_dokumenty_zrodla
WHERE dotyczy_tabela = :dotyczy_tabela
  AND dotyczy_id = :dotyczy_id
ORDER BY pole, id DESC;

-- name: wpisz_zrodlo :exec
-- Skąd pochodzi wartość pola: dokument albo osoba, potwierdzona albo nie.
INSERT INTO sprawa_dokumenty_zrodla (dotyczy_tabela, dotyczy_id, pole, dokument_id, od_uczestnik_id,
  od_kogo, potwierdzone, wpisano)
VALUES (:dotyczy_tabela, :dotyczy_id, :pole, :dokument_id, :od_uczestnik_id, :od_kogo,
  :potwierdzone, :wpisano);

-- name: potwierdz_zrodlo :exec
-- Dana od osoby została potwierdzona, np. dokumentem albo drugim kanałem.
UPDATE sprawa_dokumenty_zrodla
SET potwierdzone = 1
WHERE id = :id;

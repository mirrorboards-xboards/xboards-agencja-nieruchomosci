-- name: zgody :many
-- Zgody i oświadczenia w teczce, najnowsze pierwsze.
SELECT *
FROM sprawa_zgody_wpisy
ORDER BY zlozono DESC, id;

-- name: zgody_biezace :many
-- Zgody, których żadna nowsza nie zastąpiła.
SELECT *
FROM sprawa_zgody_wpisy AS zgoda
WHERE NOT EXISTS (
  SELECT 1
  FROM sprawa_zgody_wpisy AS nowsza
  WHERE nowsza.zastepuje_id = zgoda.id
)
ORDER BY zgoda.zlozono DESC, zgoda.id;

-- name: wpisz_zgode :exec
-- Zgoda albo oświadczenie z datą, treścią i dokumentem albo wiadomością, w której je złożono;
-- nowa zgoda w tej samej sprawie wskazuje tę, którą zastępuje.
INSERT INTO sprawa_zgody_wpisy (id, uczestnik_id, rodzaj, tresc, zlozono, dokument_id, wplyw_id,
  trwaly_nosnik, dotyczy_tabela, dotyczy_id, zastepuje_id)
VALUES (:id, :uczestnik_id, :rodzaj, :tresc, :zlozono, :dokument_id, :wplyw_id, :trwaly_nosnik,
  :dotyczy_tabela, :dotyczy_id, :zastepuje_id);

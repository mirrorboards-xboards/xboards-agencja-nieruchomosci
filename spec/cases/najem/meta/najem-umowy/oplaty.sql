-- name: oplaty_najmu :many
-- Opłaty obok czynszu z tym, kto ma umowę z dostawcą.
SELECT *
FROM najem_umowy_oplaty
ORDER BY najem_id, id;

-- name: dopisz_oplate_najmu :exec
-- Opłata z projektu umowy najmu.
INSERT INTO najem_umowy_oplaty (najem_id, rodzaj, sposob, umowa_z_dostawca_ma)
VALUES (:najem_id, :rodzaj, :sposob, :umowa_z_dostawca_ma);

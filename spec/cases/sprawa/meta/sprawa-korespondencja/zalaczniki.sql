-- name: zalaczniki :many
-- Dokumenty, które wyszły z wysyłkami albo przyszły z wpływami.
SELECT *
FROM sprawa_korespondencja_zalaczniki
ORDER BY id;

-- name: dolacz_dokument_do_wysylki :exec
-- Dokument z listy, który wychodzi razem z wysyłką, np. kopia polisy dla klienta; dołącza się
-- go tylko do projektu.
-- param wysylka_id: string
INSERT INTO sprawa_korespondencja_zalaczniki (wysylka_id, dokument_id)
VALUES (:wysylka_id, :dokument_id);

-- name: dolacz_dokument_do_wplywu :exec
-- Dokument z listy, który przyszedł z wpływem.
-- param wplyw_id: string
INSERT INTO sprawa_korespondencja_zalaczniki (wplyw_id, dokument_id)
VALUES (:wplyw_id, :dokument_id);

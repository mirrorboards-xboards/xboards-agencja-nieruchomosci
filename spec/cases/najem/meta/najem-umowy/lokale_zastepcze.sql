-- name: lokale_zastepcze :many
-- Lokale wskazane na wypadek egzekucji; bieżący nie ma niemozliwy_od.
SELECT *
FROM najem_umowy_lokale_zastepcze
ORDER BY wskazano, id;

-- name: wpisz_lokal_zastepczy :exec
-- Lokal, w którym najemca okazjonalny zamieszka w razie egzekucji.
INSERT INTO najem_umowy_lokale_zastepcze (najem_id, adres, wskazano)
VALUES (:najem_id, :adres, :wskazano);

-- name: wskaz_zgode_na_lokal_zastepczy :exec
-- Oświadczenie właściciela tamtego lokalu o zgodzie na zamieszkanie najemcy.
-- param zgoda_dokument_id: string
UPDATE najem_umowy_lokale_zastepcze
SET zgoda_dokument_id = :zgoda_dokument_id
WHERE id = :id;

-- name: odnotuj_utrate_lokalu_zastepczego :exec
-- Najemca dowiedział się, że nie może zamieszkać we wskazanym lokalu; ma 21 dni na nowy.
-- param niemozliwy_od: string
UPDATE najem_umowy_lokale_zastepcze
SET niemozliwy_od = :niemozliwy_od
WHERE id = :id;

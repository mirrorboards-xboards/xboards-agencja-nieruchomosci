-- name: ceny_nocy :many
-- Bieżące ceny nocy w każdym kanale.
SELECT *
FROM lokal_krotkoterminowy_kalendarz_ceny_biezace
ORDER BY data, kanal;

-- name: historia_cen_nocy :many
-- Wszystkie ustawione ceny w kolejności ustawienia.
SELECT *
FROM lokal_krotkoterminowy_kalendarz_ceny
ORDER BY ustawiono, rowid;

-- name: ustaw_cene_nocy :exec
-- Cena nocy w kanale z wyliczeniem programu; poza widełkami wskazuje zgodę człowieka.
-- Identyfikator (UUID) nadaje aplikacja, bo wskazuje go decyzja.
INSERT INTO lokal_krotkoterminowy_kalendarz_ceny (id, data, kanal, cena, ustawiono, wyliczenie_id,
  decyzja_id)
VALUES (:id, :data, :kanal, :cena, :ustawiono, :wyliczenie_id, :decyzja_id);

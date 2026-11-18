-- name: zgloszenia_gosci :many
-- Zgłoszenia gości w kolejności czasu.
SELECT *
FROM lokal_krotkoterminowy_pobyty_zgloszenia
ORDER BY chwila, id;

-- name: wpisz_zgloszenie_goscia :exec
-- Problem zgłoszony przez gościa z godziną i opisem; przy zagrożeniu także kogo zawiadomiono.
-- Identyfikator (UUID) nadaje aplikacja, bo wskazuje go zlecenie naprawy.
INSERT INTO lokal_krotkoterminowy_pobyty_zgloszenia (id, pobyt_id, chwila, opis, przekazano,
  zagrozenie, zawiadomiono)
VALUES (:id, :pobyt_id, :chwila, :opis, :przekazano, :zagrozenie, :zawiadomiono);

-- name: odnotuj_przekazanie_zgloszenia :exec
-- Komu Konsjerż przekazał zgłoszenie; wpisuje się raz.
-- param przekazano: string
UPDATE lokal_krotkoterminowy_pobyty_zgloszenia
SET przekazano = :przekazano
WHERE id = :id;

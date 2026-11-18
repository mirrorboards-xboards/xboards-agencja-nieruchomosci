-- name: potracenia_z_kaucji :many
-- Potrącenia z kaucji z dokumentem i zgodą.
SELECT *
FROM najem_kaucja_potracenia
ORDER BY rozliczenie_id, rowid;

-- name: dopisz_potracenie_z_kaucji :exec
-- Należność wynajmującego do potrącenia, z dokumentem, na którym stoi; identyfikator (UUID)
-- nadaje aplikacja, bo wskazuje go decyzja.
INSERT INTO najem_kaucja_potracenia (id, rozliczenie_id, tytul, kwota, dokument_id)
VALUES (:id, :rozliczenie_id, :tytul, :kwota, :dokument_id);

-- name: wskaz_zgode_na_potracenie :exec
-- Zatwierdzona decyzja inna o tym potrąceniu.
-- param decyzja_id: string
UPDATE najem_kaucja_potracenia
SET decyzja_id = :decyzja_id
WHERE id = :id;

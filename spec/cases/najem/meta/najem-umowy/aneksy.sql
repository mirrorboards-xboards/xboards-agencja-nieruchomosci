-- name: aneksy_najmu :many
-- Aneksy umów najmu; obowiązujące mają obowiazuje_od.
SELECT *
FROM najem_umowy_aneksy
ORDER BY rowid;

-- name: przygotuj_aneks_najmu :exec
-- Projekt aneksu zawartej umowy najmu; identyfikator (UUID) nadaje aplikacja, bo wskazuje go
-- decyzja.
INSERT INTO najem_umowy_aneksy (id, najem_id, opis, projekt_dokument_id)
VALUES (:id, :najem_id, :opis, :projekt_dokument_id);

-- name: wskaz_zgode_na_aneks_najmu :exec
-- Zatwierdzona decyzja inna o przekazaniu aneksu stronom.
-- param decyzja_id: string
UPDATE najem_umowy_aneksy
SET decyzja_id = :decyzja_id
WHERE id = :id;

-- name: odnotuj_podpisanie_aneksu_najmu :exec
-- Strony podpisały aneks; od tego dnia obowiązuje.
-- param dokument_id: string
-- param obowiazuje_od: string
UPDATE najem_umowy_aneksy
SET dokument_id = :dokument_id,
  obowiazuje_od = :obowiazuje_od
WHERE id = :id;

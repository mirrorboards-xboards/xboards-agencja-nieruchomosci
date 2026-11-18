-- name: rachunki :many
-- Rachunki uczestników, także zastąpione, w kolejności wpisania.
SELECT *
FROM sprawa_rachunki_wpisy
ORDER BY wpisano, id;

-- name: rachunki_biezace :many
-- Rachunki, których nikt nie zastąpił.
SELECT *
FROM sprawa_rachunki_wpisy
WHERE zastapiony_przez_id IS NULL
ORDER BY wpisano, id;

-- name: rachunki_dopuszczone :many
-- Rachunki, na które wolno płacić: niezastąpione i dopuszczone zatwierdzoną decyzją.
SELECT *
FROM sprawa_rachunki_dopuszczone
ORDER BY wpisano, id;

-- name: wpisz_rachunek :exec
-- Nowy numer rachunku z jego źródłem; czeka na potwierdzenie, sprawdzenie i zgodę.
INSERT INTO sprawa_rachunki_wpisy (id, uczestnik_id, numer, posiadacz, cel, zrodlo_dokument_id,
  zrodlo_wplyw_id, wpisano)
VALUES (:id, :uczestnik_id, :numer, :posiadacz, :cel, :zrodlo_dokument_id, :zrodlo_wplyw_id,
  :wpisano);

-- name: odnotuj_potwierdzenie_rachunku :exec
-- Posiadacz potwierdzony innym kanałem niż ten, którym przyszedł numer.
-- param potwierdzil: string
-- param potwierdzono: string
-- param potwierdzenie_sposob: string
UPDATE sprawa_rachunki_wpisy
SET potwierdzil = :potwierdzil,
  potwierdzono = :potwierdzono,
  potwierdzenie_sposob = :potwierdzenie_sposob
WHERE id = :id;

-- name: zastap_rachunek :exec
-- Stary rachunek zastąpiony nowym wierszem, z tym, kto i kiedy go zastąpił.
-- param zastapiony_przez_id: string
-- param zastapil: string
-- param zastapiono: string
UPDATE sprawa_rachunki_wpisy
SET zastapiony_przez_id = :zastapiony_przez_id,
  zastapil = :zastapil,
  zastapiono = :zastapiono
WHERE id = :id;

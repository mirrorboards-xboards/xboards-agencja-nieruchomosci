-- name: watpliwosci :many
-- Znaczniki wątpliwości Kontrolerów, stojące i zdjęte, w kolejności postawienia.
SELECT *
FROM sprawa_metryka_watpliwosci
ORDER BY postawiono, id;

-- name: watpliwosci_stojace :many
-- Znaczniki, które jeszcze stoją; kierownik działu widzi je bez powodu.
SELECT *
FROM sprawa_metryka_watpliwosci
WHERE zdjecie_decyzja_id IS NULL
ORDER BY postawiono, id;

-- name: postaw_watpliwosc :exec
-- Kontroler stawia własny znacznik, bez powodu (powód zostaje w rejestrze kontroli).
INSERT INTO sprawa_metryka_watpliwosci (id, postawil, postawiono)
VALUES (:id, :postawil, :postawiono);

-- name: zdejmij_watpliwosc :exec
-- Zdjęcie znacznika na podstawie zatwierdzonej decyzji zdjecie_watpliwosci, która go wskazuje.
-- param zdjecie_decyzja_id: string
UPDATE sprawa_metryka_watpliwosci
SET zdjecie_decyzja_id = :zdjecie_decyzja_id
WHERE id = :id;

-- name: reguly_przydzialu :many
-- Aktualne reguły przydziału według rodzaju zgłoszenia.
SELECT *
FROM kierownik_biura_przydzial_reguly
WHERE zastapiona_od IS NULL
ORDER BY rodzaj_zgloszenia, id;

-- name: historia_regul :many
-- Wszystkie reguły przydziału, także zastąpione, od najstarszej; z nich kierownik co tydzień
-- porównuje przydziały z regułami (dryf).
SELECT *
FROM kierownik_biura_przydzial_reguly
ORDER BY od, id;

-- name: zapisz_regule_przydzialu :exec
-- Nowa reguła podpisana przez właściciela biura; poprzednią dla tego rodzaju trzeba najpierw
-- zastąpić.
INSERT INTO kierownik_biura_przydzial_reguly (rodzaj_zgloszenia, dzial, od, podpisal, podpis)
VALUES (:rodzaj_zgloszenia, :dzial, :od, :podpisal, :podpis);

-- name: zastap_regule_przydzialu :exec
-- Reguła przestaje obowiązywać, bo właściciel biura podpisał nową.
-- param zastapiona_od: string
UPDATE kierownik_biura_przydzial_reguly
SET zastapiona_od = :zastapiona_od
WHERE id = :id;

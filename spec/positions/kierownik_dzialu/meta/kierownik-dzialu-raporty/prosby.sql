-- name: prosby_na_liscie :many
-- Prośby przekazane dalej, których zdjęcia kierownik jeszcze nie zgłosił, od najstarszej.
SELECT *
FROM kierownik_dzialu_raporty_prosby
WHERE zgloszono_zdjecie IS NULL
ORDER BY przekazano, id;

-- name: przekaz_prosbe :exec
-- Prośba z raportu przekazana Kierownikowi biura albo, w Kontroli, właścicielowi.
INSERT INTO kierownik_dzialu_raporty_prosby (decyzja_odnosnik, raport_id, przekazano_do, przekazano)
VALUES (:decyzja_odnosnik, :raport_id, :przekazano_do, :przekazano);

-- name: zglos_zdjecie_prosby :exec
-- Prośbę rozstrzygnięto albo przestała być potrzebna, więc kierownik zgłasza, że można ją zdjąć
-- z listy.
-- param zgloszono_zdjecie: string
-- param powod_zdjecia: "rozstrzygnieta" | "zbedna"
UPDATE kierownik_dzialu_raporty_prosby
SET zgloszono_zdjecie = :zgloszono_zdjecie,
  powod_zdjecia = :powod_zdjecia
WHERE id = :id;

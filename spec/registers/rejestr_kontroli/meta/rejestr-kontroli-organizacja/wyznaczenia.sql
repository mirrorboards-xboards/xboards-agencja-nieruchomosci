-- name: wyznaczenia :many
-- Wyznaczenia osób z art. 6 i 8 AML, bieżące i zakończone.
SELECT *
FROM rejestr_kontroli_organizacja_wyznaczenia
ORDER BY funkcja, od DESC, id;

-- name: wpisz_wyznaczenie :exec
-- Osoba wyznaczona do funkcji od dnia.
INSERT INTO rejestr_kontroli_organizacja_wyznaczenia (funkcja, osoba, od, podstawa, wpisal, wpisano)
VALUES (:funkcja, :osoba, :od, :podstawa, :wpisal, :wpisano);

-- name: zakoncz_wyznaczenie :exec
-- Koniec wyznaczenia; wpis zostaje.
-- param do: string
UPDATE rejestr_kontroli_organizacja_wyznaczenia
SET do = :do
WHERE id = :id;

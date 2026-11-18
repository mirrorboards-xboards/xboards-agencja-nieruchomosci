-- name: osoby_wykonawcow :many
-- Bieżący pracownicy wykonawców. Koordynator porównuje z nimi nadawcę wiadomości i wysyła kod do
-- zamka tylko osobie z kody_do_zamkow = 1.
SELECT *
FROM wykonawcy_spis_osoby
WHERE do IS NULL
ORDER BY wykonawca_id, id;

-- name: osoby_wykonawcy :many
-- Pracownicy jednego wykonawcy, bieżący i zakończeni.
SELECT *
FROM wykonawcy_spis_osoby
WHERE wykonawca_id = :wykonawca_id
ORDER BY id DESC;

-- name: wpisz_osobe_wykonawcy :exec
-- Pracownik wykonawcy po decyzji o przyjęciu albo zmianie danych.
INSERT INTO wykonawcy_spis_osoby (wykonawca_id, imiona, nazwisko, telefon, kody_do_zamkow, od,
  decyzja_id)
VALUES (:wykonawca_id, :imiona, :nazwisko, :telefon, :kody_do_zamkow, :od, :decyzja_id);

-- name: zakoncz_osobe_wykonawcy :exec
-- Pracownik, który przestał pracować dla biura, po decyzji o zmianie danych albo wykreśleniu.
-- param zakonczenie_decyzja_id: string
UPDATE wykonawcy_spis_osoby
SET do = :do,
  zakonczenie_decyzja_id = :zakonczenie_decyzja_id
WHERE id = :id;

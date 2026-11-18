-- name: ogloszenia :many
-- Ogłoszenia oferty w kolejności wpisania.
SELECT *
FROM oferta_ogloszenia_wpisy
ORDER BY rowid;

-- name: przygotuj_ogloszenie :exec
-- Projekt ogłoszenia w jednym kanale; identyfikator (UUID) nadaje aplikacja, bo wskazują go
-- wersje, decyzja o pierwszej publikacji i katalog ofert.
INSERT INTO oferta_ogloszenia_wpisy (id, dotyczy, kanal, portal)
VALUES (:id, :dotyczy, :kanal, :portal);

-- name: odnotuj_publikacje_ogloszenia :exec
-- Ogłoszenie wyszło: najnowsza wersja przechodzi reguły publikacji, a pierwsza publikacja
-- w teczce ma zatwierdzoną decyzję pośrednika.
-- param opublikowano: string
UPDATE oferta_ogloszenia_wpisy
SET opublikowano = :opublikowano
WHERE id = :id;

-- name: odnotuj_numer_ogloszenia :exec
-- Numer nadany przez portal; Kierownik pośrednictwa przepisuje go do katalogu ofert.
-- param numer_ogloszenia: string
UPDATE oferta_ogloszenia_wpisy
SET numer_ogloszenia = :numer_ogloszenia
WHERE id = :id;

-- name: zdejmij_ogloszenie :exec
-- Zdjęcie ogłoszenia z powodem, np. przyjęta oferta albo koniec umowy pośrednictwa; zdjęte
-- ogłoszenie nie wraca.
-- param zdjeto: string
-- param powod_zdjecia: string
UPDATE oferta_ogloszenia_wpisy
SET zdjeto = :zdjeto,
  powod_zdjecia = :powod_zdjecia
WHERE id = :id;

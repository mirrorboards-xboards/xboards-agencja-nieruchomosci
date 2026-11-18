-- name: lista_decyzji :many
-- Lista jednego człowieka: najpierw pozycje z terminem nie późniejszym niż `bliski_termin`, według
-- terminu, potem pozostałe od najstarszej.
-- param bliski_termin: string
SELECT *
FROM kierownik_biura_decyzje_pozycje
WHERE decyduje = :decyduje
  AND zdjeto IS NULL
ORDER BY (termin IS NOT NULL AND termin <= :bliski_termin) DESC,
  CASE
    WHEN termin IS NOT NULL AND termin <= :bliski_termin THEN termin
  END,
  przyjeto,
  id;

-- name: pozycje_czekajace_od :many
-- Niezdjęte pozycje przyjęte przed jedną chwilą, od najstarszej; to decyzje, które czekają za długo.
-- param przed: string
SELECT *
FROM kierownik_biura_decyzje_pozycje
WHERE zdjeto IS NULL
  AND przyjeto < :przed
ORDER BY przyjeto, id;

-- name: pozycje_zdjete_od :many
-- Pozycje zdjęte od jednego dnia; z nich widać czas oczekiwania decyzji w sprawozdaniu.
-- param od: string
SELECT *
FROM kierownik_biura_decyzje_pozycje
WHERE zdjeto >= :od
ORDER BY zdjeto, id;

-- name: przyjmij_pozycje :exec
-- Nowa pozycja z prośby kierownika działu.
INSERT INTO kierownik_biura_decyzje_pozycje (prosba_odnosnik, dzial, decyduje, opis, termin,
  przyjeto)
VALUES (:prosba_odnosnik, :dzial, :decyduje, :opis, :termin, :przyjeto);

-- name: odnow_pozycje :exec
-- Ta sama prośba przyszła ponownie: pozycja dostaje nowy opis i termin, ale się nie podwaja.
-- param odnowiono: string
UPDATE kierownik_biura_decyzje_pozycje
SET opis = :opis,
  termin = :termin,
  odnowiono = :odnowiono
WHERE id = :id;

-- name: odnotuj_wyslanie_pozycji :exec
-- Pozycja pierwszy raz trafiła do człowieka na jego liście.
-- param trafila_do_czlowieka: string
UPDATE kierownik_biura_decyzje_pozycje
SET trafila_do_czlowieka = :trafila_do_czlowieka
WHERE id = :id;

-- name: odnotuj_osobne_zgloszenie :exec
-- Decyzję, która czeka za długo, kierownik zgłosił człowiekowi osobno.
-- param zgloszono_osobno: string
UPDATE kierownik_biura_decyzje_pozycje
SET zgloszono_osobno = :zgloszono_osobno
WHERE id = :id;

-- name: zdejmij_pozycje :exec
-- Kierownik działu zgłosił, że prośba jest rozstrzygnięta albo zbędna.
-- param zdjeto: string
-- param powod_zdjecia: "rozstrzygnieta" | "zbedna"
UPDATE kierownik_biura_decyzje_pozycje
SET zdjeto = :zdjeto,
  powod_zdjecia = :powod_zdjecia
WHERE id = :id;

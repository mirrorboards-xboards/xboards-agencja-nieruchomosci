-- name: prezentacje :many
-- Prezentacje oferty w kolejności terminów.
SELECT *
FROM oferta_zainteresowani_prezentacje
ORDER BY termin, id;

-- name: uczestnicy_prezentacji :many
-- Uczestnicy prezentacji z obecnością i chwilą podania pełnego adresu.
SELECT *
FROM oferta_zainteresowani_prezentacje_uczestnicy
ORDER BY prezentacja_id, id;

-- name: umow_prezentacje :exec
-- Prezentacja w kalendarzu pośrednika, nie wcześniej niż praca według umowy pośrednictwa;
-- identyfikator (UUID) nadaje aplikacja, bo wskazują go uczestnicy.
INSERT INTO oferta_zainteresowani_prezentacje (id, termin, posrednik, kalendarz_odnosnik, otwiera)
VALUES (:id, :termin, :posrednik, :kalendarz_odnosnik, :otwiera);

-- name: dopisz_uczestnika_prezentacji :exec
-- Osoba umówiona na prezentację; chwila podania pełnego adresu, gdy Opiekun już go podał.
INSERT INTO oferta_zainteresowani_prezentacje_uczestnicy (prezentacja_id, osoba_id, adres_podano)
VALUES (:prezentacja_id, :osoba_id, :adres_podano);

-- name: odnotuj_podanie_adresu :exec
-- Chwila, w której Opiekun podał uczestnikowi pełny adres; wpisuje się ją raz.
-- param adres_podano: string
UPDATE oferta_zainteresowani_prezentacje_uczestnicy
SET adres_podano = :adres_podano
WHERE id = :id;

-- name: odnotuj_obecnosc_na_prezentacji :exec
-- Czy uczestnik przyszedł na prezentację.
-- param przyszedl: boolean
UPDATE oferta_zainteresowani_prezentacje_uczestnicy
SET przyszedl = :przyszedl
WHERE id = :id;

-- name: odnotuj_przebieg_prezentacji :exec
-- Notatka pośrednika po prezentacji, spisana przez Opiekuna.
-- param przebieg: string
UPDATE oferta_zainteresowani_prezentacje
SET przebieg = :przebieg
WHERE id = :id;

-- name: odwolaj_prezentacje :exec
-- Kto, kiedy i dlaczego odwołał prezentację.
-- param odwolano: string
-- param odwolal: string
-- param powod_odwolania: string
UPDATE oferta_zainteresowani_prezentacje
SET odwolano = :odwolano,
  odwolal = :odwolal,
  powod_odwolania = :powod_odwolania
WHERE id = :id;

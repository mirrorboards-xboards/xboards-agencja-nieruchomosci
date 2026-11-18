-- name: prezentacje_propozycji :many
-- Prezentacje oglądanych nieruchomości w kolejności terminów.
SELECT *
FROM poszukiwanie_propozycje_prezentacje
ORDER BY termin, id;

-- name: pytania_otwarte :many
-- Pytania klienta do strony, która pokazuje nieruchomość, bez odpowiedzi.
SELECT *
FROM poszukiwanie_propozycje_pytania
WHERE odpowiedz IS NULL
ORDER BY prezentacja_id, id;

-- name: umow_prezentacje_propozycji :exec
-- Prezentacja propozycji, którą klient dostał, z danymi klienta podanymi drugiej stronie;
-- szerszy zakres wskazuje zatwierdzoną decyzję człowieka. Identyfikator (UUID) nadaje aplikacja,
-- bo wskazują go pytania i decyzja.
INSERT INTO poszukiwanie_propozycje_prezentacje (id, propozycja_id, termin, kto_pokazuje,
  dane_klienta_podane, szerszy_zakres, szerszy_zakres_decyzja_id, uczestnicy)
VALUES (:id, :propozycja_id, :termin, :kto_pokazuje, :dane_klienta_podane, :szerszy_zakres,
  :szerszy_zakres_decyzja_id, :uczestnicy);

-- name: odnotuj_opinie_klienta :exec
-- Co klientowi się podobało, co nie i czy chce złożyć ofertę; wpisuje się ją raz.
-- param opinia_klienta: string
UPDATE poszukiwanie_propozycje_prezentacje
SET opinia_klienta = :opinia_klienta
WHERE id = :id;

-- name: odnotuj_notatke_posrednika :exec
-- Co pośrednik zauważył na miejscu; wpisuje się ją raz.
-- param notatka_posrednika: string
UPDATE poszukiwanie_propozycje_prezentacje
SET notatka_posrednika = :notatka_posrednika
WHERE id = :id;

-- name: odwolaj_prezentacje_propozycji :exec
-- Kto, kiedy i dlaczego odwołał prezentację.
-- param odwolano: string
-- param odwolal: string
-- param powod_odwolania: string
UPDATE poszukiwanie_propozycje_prezentacje
SET odwolano = :odwolano,
  odwolal = :odwolal,
  powod_odwolania = :powod_odwolania
WHERE id = :id;

-- name: dopisz_pytanie_do_prezentacji :exec
-- Pytanie, o które klient chce dopytać drugą stronę.
INSERT INTO poszukiwanie_propozycje_pytania (prezentacja_id, pytanie)
VALUES (:prezentacja_id, :pytanie);

-- name: odnotuj_odpowiedz_na_pytanie :exec
-- Odpowiedź drugiej strony z wpływem, w którym przyszła; wpisuje się ją raz.
-- param odpowiedz: string
-- param odpowiedz_wplyw_id: string
UPDATE poszukiwanie_propozycje_pytania
SET odpowiedz = :odpowiedz,
  odpowiedz_wplyw_id = :odpowiedz_wplyw_id
WHERE id = :id;

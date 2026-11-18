-- name: zestawienia :many
-- Zestawienia dla właściciela w kolejności miesięcy.
SELECT *
FROM lokal_krotkoterminowy_zestawienia_wpisy
ORDER BY miesiac;

-- name: przygotuj_zestawienie :exec
-- Projekt zestawienia za miesiąc; korekta wcześniejszego zestawienia wskazuje je i zgodę
-- człowieka. Identyfikator (UUID) nadaje aplikacja, bo wskazują go pozycje, decyzja i wypłata.
INSERT INTO lokal_krotkoterminowy_zestawienia_wpisy (id, miesiac, utworzono, korekta_do_id,
  korekta_decyzja_id, stan)
VALUES (:id, :miesiac, :utworzono, :korekta_do_id, :korekta_decyzja_id, 'projekt');

-- name: zamknij_zestawienie :exec
-- Projekt gotowy do zgody: plik z programu, kwota do wypłaty (suma pozycji) i rezerwa po
-- zestawieniu.
-- param sciezka: string
-- param skrot: string
-- param kwota_do_wyplaty: integer
-- param rezerwa_po: integer
UPDATE lokal_krotkoterminowy_zestawienia_wpisy
SET stan = 'gotowe',
  sciezka = :sciezka,
  skrot = :skrot,
  kwota_do_wyplaty = :kwota_do_wyplaty,
  rezerwa_po = :rezerwa_po
WHERE id = :id;

-- name: wskaz_wysylke_zestawienia :exec
-- Wysyłka gotowego zestawienia, zanim wyjdzie; czeka ona na zgodę człowieka.
-- param wysylka_id: string
UPDATE lokal_krotkoterminowy_zestawienia_wpisy
SET wysylka_id = :wysylka_id
WHERE id = :id;

-- name: cofnij_zestawienie_do_projektu :exec
-- Gotowe zestawienie wraca do poprawki, gdy nie ma wysyłki albo jego wysyłkę anulowano.
UPDATE lokal_krotkoterminowy_zestawienia_wpisy
SET stan = 'projekt',
  wysylka_id = NULL
WHERE id = :id;

-- name: odnotuj_wyslanie_zestawienia :exec
-- Zestawienie wyszło do właściciela; od tej chwili się nie zmienia.
UPDATE lokal_krotkoterminowy_zestawienia_wpisy
SET stan = 'wyslane'
WHERE id = :id;

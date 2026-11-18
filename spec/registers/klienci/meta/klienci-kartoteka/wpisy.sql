-- name: klienci :many
-- Klienci kartoteki z aktualną wersją danych, w kolejności wpisania.
SELECT
  wpisy.id,
  wpisy.rodzaj_podmiotu,
  wpisy.wpisano,
  dane.imiona,
  dane.nazwisko,
  dane.nazwa,
  dane.firma
FROM klienci_kartoteka_wpisy AS wpisy
JOIN klienci_kartoteka_dane AS dane ON dane.klient_id = wpisy.id
  AND dane.aktualne = 1
ORDER BY wpisy.rowid;

-- name: klienci_po_nazwisku_albo_nazwie :many
-- Klienci, których którakolwiek wersja danych ma to nazwisko albo tę nazwę. Kierownik biura
-- porównuje kartę Recepcji także z danymi sprzed zmiany, np. z nazwiskiem sprzed ślubu.
-- param szukane: string
SELECT DISTINCT
  dane.klient_id,
  dane.imiona,
  dane.nazwisko,
  dane.nazwa,
  dane.od,
  dane.aktualne
FROM klienci_kartoteka_dane AS dane
WHERE dane.nazwisko = :szukane
  OR dane.nazwa = :szukane
ORDER BY dane.klient_id, dane.id;

-- name: wpisz_klienta :exec
-- Nowy klient z pierwszą wersją danych, w dniu podpisania pierwszej umowy. Identyfikator (UUID)
-- nadaje aplikacja, bo wskazują go umowy, adresy i kontakty klienta.
INSERT INTO klienci_kartoteka_wpisy (id, rodzaj_podmiotu, wpisal, wpisano)
VALUES (:id, :rodzaj_podmiotu, :wpisal, :wpisano);
INSERT INTO klienci_kartoteka_dane (klient_id, od, aktualne, imiona, nazwisko, pesel, nazwa,
  forma_prawna, krs, ceidg, nip, regon, firma, zrodlo_odnosnik, zrodlo_opis, wpisal)
VALUES (:id, :od, 1, :imiona, :nazwisko, :pesel, :nazwa, :forma_prawna, :krs, :ceidg, :nip, :regon,
  :firma, :zrodlo_odnosnik, :zrodlo_opis, :wpisal);

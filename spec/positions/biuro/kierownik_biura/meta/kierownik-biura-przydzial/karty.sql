-- name: karty_do_rozstrzygniecia :many
-- Karty, które czekają na kierownika albo na zgodę człowieka, od najstarszej.
SELECT *
FROM kierownik_biura_przydzial_karty
WHERE stan IN ('czeka', 'u_czlowieka')
ORDER BY wplynela, id;

-- name: karty_rozstrzygniete_od :many
-- Karty przydzielone i odrzucone od jednego dnia, do sprawozdania i do porównania z regułami.
-- param od: string
SELECT *
FROM kierownik_biura_przydzial_karty
WHERE stan IN ('przydzielona', 'odrzucona')
  AND rozstrzygnieto >= :od
ORDER BY rozstrzygnieto, id;

-- name: przyjmij_karte :exec
-- Karta od Recepcji ze stałymi polami.
INSERT INTO kierownik_biura_przydzial_karty (karta_odnosnik, wplynela, rodzaj_karty, rodzaj_sprawy,
  rodzaj_nieruchomosci, miejscowosc, kanal, kontakt, stan)
VALUES (:karta_odnosnik, :wplynela, :rodzaj_karty, :rodzaj_sprawy, :rodzaj_nieruchomosci,
  :miejscowosc, :kanal, :kontakt, 'czeka');

-- name: odnotuj_sprawdzenie_kartoteki :exec
-- Wynik sprawdzenia zgłaszającego w kartotece klientów.
-- param kartoteka_wynik: "brak" | "jeden" | "kilka_albo_czesciowo"
-- param sprawdzono_kartoteke: string
UPDATE kierownik_biura_przydzial_karty
SET kartoteka_wynik = :kartoteka_wynik,
  kartoteka_odnosnik = :kartoteka_odnosnik,
  sprawdzono_kartoteke = :sprawdzono_kartoteke
WHERE id = :id;

-- name: przekaz_karte_czlowiekowi :exec
-- Karta czeka na zgodę człowieka, np. bo reguły jej nie rozstrzygają.
UPDATE kierownik_biura_przydzial_karty
SET stan = 'u_czlowieka'
WHERE id = :id;

-- name: przydziel_karte :exec
-- Karta przydzielona działowi według reguły albo za zgodą człowieka; kontakt trafił do działu,
-- więc znika z karty.
-- param dzial: "posrednictwo" | "transakcje" | "najem"
-- param rozstrzygnieto: string
UPDATE kierownik_biura_przydzial_karty
SET stan = 'przydzielona',
  dzial = :dzial,
  regula_id = :regula_id,
  rozstrzygnieto = :rozstrzygnieto,
  zatwierdzil = :zatwierdzil,
  podpis = :podpis,
  zatwierdzono = :zatwierdzono,
  kontakt = NULL
WHERE id = :id;

-- name: odrzuc_karte :exec
-- Karta odrzucona z powodem, np. reklama; odrzucenie zgłoszenia klienta wymaga zgody człowieka.
-- param powod: string
-- param rozstrzygnieto: string
UPDATE kierownik_biura_przydzial_karty
SET stan = 'odrzucona',
  powod = :powod,
  rozstrzygnieto = :rozstrzygnieto,
  zatwierdzil = :zatwierdzil,
  podpis = :podpis,
  zatwierdzono = :zatwierdzono,
  kontakt = NULL
WHERE id = :id;

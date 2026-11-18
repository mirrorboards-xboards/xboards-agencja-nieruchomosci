-- name: klienci_kontroli :many
-- Stosunki gospodarcze z klientami, najnowsze pierwsze; klient, który wrócił, ma kilka wpisów.
SELECT *
FROM rejestr_kontroli_klienci_wpisy
ORDER BY wpisano DESC, id;

-- name: stosunki_klienta_z_kartoteki :many
-- Wszystkie stosunki klienta z kartoteki; Kierownik kontroli wskazuje dotychczasowego Kontrolera
-- klienta, który wraca (R7).
SELECT *
FROM rejestr_kontroli_klienci_wpisy
WHERE kartoteka_odnosnik = :kartoteka_odnosnik
ORDER BY wpisano, id;

-- name: wpisz_klienta_kontroli :exec
-- Klient z Kontrolerem przy weryfikacji przed pierwszą umową. Identyfikator (UUID) nadaje
-- aplikacja.
INSERT INTO rejestr_kontroli_klienci_wpisy (id, klient, kartoteka_odnosnik, kontroler, stosunki_od,
  stosunki_do, zrodlo, wpisal, wpisano)
VALUES (:id, :klient, :kartoteka_odnosnik, :kontroler, :stosunki_od, :stosunki_do, :zrodlo, :wpisal,
  :wpisano);

-- name: wpisz_odnosnik_do_kartoteki :exec
-- Wpis klienta w kartotece, gdy powstał w dniu podpisania umowy.
-- param kartoteka_odnosnik: string
UPDATE rejestr_kontroli_klienci_wpisy
SET kartoteka_odnosnik = :kartoteka_odnosnik
WHERE id = :id;

-- name: wpisz_nawiazanie_stosunkow :exec
-- Dzień podpisania pierwszej umowy z biurem.
-- param stosunki_od: string
UPDATE rejestr_kontroli_klienci_wpisy
SET stosunki_od = :stosunki_od
WHERE id = :id;

-- name: wpisz_zakonczenie_stosunkow :exec
-- Dzień zakończenia stosunków z raportu końcowego Kontrolera; od niego biegnie 5 lat
-- przechowywania.
-- param stosunki_do: string
UPDATE rejestr_kontroli_klienci_wpisy
SET stosunki_do = :stosunki_do
WHERE id = :id;

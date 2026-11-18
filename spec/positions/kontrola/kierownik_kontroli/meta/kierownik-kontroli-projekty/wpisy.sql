-- name: projekty_w_toku :many
-- Projekty, które jeszcze nie zostały przekazane ani odstąpione, od najbliższego terminu.
SELECT *
FROM kierownik_kontroli_projekty_wpisy
WHERE stan IN ('projekt', 'u_wlasciciela')
ORDER BY termin, id;

-- name: projekty :many
-- Wszystkie projekty, najnowsze pierwsze.
SELECT *
FROM kierownik_kontroli_projekty_wpisy
ORDER BY utworzono DESC, id;

-- name: utworz_projekt :exec
-- Nowy projekt zawiadomienia z plikiem w XGit pokoju pamięci.
INSERT INTO kierownik_kontroli_projekty_wpisy (rodzaj, teczka_odnosnik, termin, utworzono, stan,
  sciezka, skrot)
VALUES (:rodzaj, :teczka_odnosnik, :termin, :utworzono, 'projekt', :sciezka, :skrot);

-- name: zmien_projekt :exec
-- Nowa wersja pliku projektu albo nowy termin, póki projekt jest w toku.
-- param sciezka: string
-- param skrot: string
UPDATE kierownik_kontroli_projekty_wpisy
SET termin = :termin,
  sciezka = :sciezka,
  skrot = :skrot
WHERE id = :id;

-- name: daj_projekt_wlascicielowi :exec
-- Projekt trafił do właściciela, który potwierdza podejrzenie i przekazuje zawiadomienie.
-- param u_wlasciciela_od: string
UPDATE kierownik_kontroli_projekty_wpisy
SET stan = 'u_wlasciciela',
  u_wlasciciela_od = :u_wlasciciela_od
WHERE id = :id;

-- name: cofnij_projekt_do_poprawy :exec
-- Właściciel oddał projekt do poprawy.
UPDATE kierownik_kontroli_projekty_wpisy
SET stan = 'projekt'
WHERE id = :id;

-- name: odnotuj_przekazanie_zawiadomienia :exec
-- Właściciel przekazał zawiadomienie; ślad jest w rejestrze kontroli, a projekt traci plik i tylko
-- go wskazuje. Plik w XGit pokoju usuwa aplikacja.
-- param przekazano: string
-- param rejestr_odnosnik: string
UPDATE kierownik_kontroli_projekty_wpisy
SET stan = 'przekazany',
  przekazano = :przekazano,
  rejestr_odnosnik = :rejestr_odnosnik,
  sciezka = NULL,
  skrot = NULL
WHERE id = :id;

-- name: odnotuj_odstapienie :exec
-- Właściciel postanowił projektu nie przekazywać i podpisał to.
-- param odstapil: string
-- param odstapienie_podpis: string
-- param odstapiono: string
UPDATE kierownik_kontroli_projekty_wpisy
SET stan = 'odstapiono',
  odstapil = :odstapil,
  odstapienie_podpis = :odstapienie_podpis,
  odstapiono = :odstapiono
WHERE id = :id;

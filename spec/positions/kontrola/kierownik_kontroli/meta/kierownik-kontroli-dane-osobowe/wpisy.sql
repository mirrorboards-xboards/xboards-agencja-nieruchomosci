-- name: sprawy_danych_osobowych :many
-- Sprawy niezałatwione od najbliższego terminu, potem załatwione od najnowszej.
SELECT *
FROM kierownik_kontroli_dane_osobowe_wpisy
ORDER BY stan = 'zalatwiona', CASE WHEN stan <> 'zalatwiona' THEN termin END, wplynelo DESC, id;

-- name: zapisz_sprawe_danych_osobowych :exec
-- Nowa prośba osoby o jej dane albo zgłoszone naruszenie, z terminem z programu.
INSERT INTO kierownik_kontroli_dane_osobowe_wpisy (rodzaj, wplynelo, wplyw_odnosnik, termin, stan)
VALUES (:rodzaj, :wplynelo, :wplyw_odnosnik, :termin, 'otwarta');

-- name: daj_sprawe_wlascicielowi :exec
-- Odpowiedź albo opis naruszenia czeka u właściciela.
UPDATE kierownik_kontroli_dane_osobowe_wpisy
SET stan = 'u_wlasciciela'
WHERE id = :id;

-- name: odnotuj_zalatwienie :exec
-- Właściciel wysłał odpowiedź albo zgłoszenie naruszenia.
-- param odpowiedz_odnosnik: string
-- param zalatwiono: string
UPDATE kierownik_kontroli_dane_osobowe_wpisy
SET stan = 'zalatwiona',
  odpowiedz_odnosnik = :odpowiedz_odnosnik,
  zalatwiono = :zalatwiono
WHERE id = :id;

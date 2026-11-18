-- name: zawiadomienia :many
-- Ślad zawiadomień, najnowsze pierwsze.
SELECT *
FROM rejestr_kontroli_zawiadomienia_wpisy
ORDER BY przekazano DESC, id;

-- name: wpisz_zawiadomienie :exec
-- Ślad zawiadomienia w dniu, w którym osoba wyznaczona je przekazała. Identyfikator (UUID) nadaje
-- aplikacja.
INSERT INTO rejestr_kontroli_zawiadomienia_wpisy (id, rodzaj, potwierdzenie_decyzja_id,
  przekazanie_decyzja_id, przekazal, przekazano, podstawy, wpisal, wpisano)
VALUES (:id, :rodzaj, :potwierdzenie_decyzja_id, :przekazanie_decyzja_id, :przekazal, :przekazano,
  :podstawy, :wpisal, :wpisano);

-- name: wpisz_potwierdzenie_przyjecia :exec
-- Urzędowe poświadczenie odbioru zawiadomienia; wpisuje się raz.
-- param potwierdzenie_przyjecia: string
UPDATE rejestr_kontroli_zawiadomienia_wpisy
SET potwierdzenie_przyjecia = :potwierdzenie_przyjecia
WHERE id = :id;

-- name: wpisz_odpowiedz_organu :exec
-- Odpowiedź organu, np. żądanie wstrzymania transakcji; wpisuje się raz.
-- param odpowiedz_organu: string
UPDATE rejestr_kontroli_zawiadomienia_wpisy
SET odpowiedz_organu = :odpowiedz_organu
WHERE id = :id;

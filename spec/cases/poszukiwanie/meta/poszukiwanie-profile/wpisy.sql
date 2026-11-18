-- name: profile_poszukiwania :many
-- Profile poszukiwania w teczce; kilka, gdy klient kupuje kilka nieruchomości.
SELECT *
FROM poszukiwanie_profile_wpisy
ORDER BY rowid;

-- name: zaloz_profil_poszukiwania :exec
-- Nowy profil szukanej nieruchomości; identyfikator (UUID) nadaje aplikacja, bo wskazują go
-- wersje.
INSERT INTO poszukiwanie_profile_wpisy (id, nazwa)
VALUES (:id, :nazwa);

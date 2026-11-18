-- name: stan_pokoju :one
-- Czy pokój jest otwarty, czy zamknięty; zamyka go i otwiera ponownie zgoda obu stron.
-- column stan: "otwarty" | "zamkniety"
SELECT stan
FROM wymiana_metryka_stan;

-- name: zamkniecia :many
-- Dziennik zamknięć i ponownych otwarć obu stron, z datami.
SELECT *
FROM wymiana_metryka_zamkniecia
ORDER BY id;

-- name: zamknij_pokoj_wymiany :exec
-- Strona zgadza się na zamknięcie pokoju; pokój zamyka się, gdy zgodzą się obie.
INSERT INTO wymiana_metryka_zamkniecia (strona_pokoju, rodzaj, chwila)
VALUES (:strona_pokoju, 'zamkniecie', :chwila);

-- name: otworz_ponownie_pokoj_wymiany :exec
-- Strona zgadza się na ponowne otwarcie zamkniętego pokoju; otwiera się, gdy zgodzą się obie.
INSERT INTO wymiana_metryka_zamkniecia (strona_pokoju, rodzaj, chwila)
VALUES (:strona_pokoju, 'ponowne_otwarcie', :chwila);

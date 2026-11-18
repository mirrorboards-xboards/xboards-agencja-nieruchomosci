-- name: zrodla :many
-- Wszystkie źródła agenta; najpierw te, którym nie można ufać.
SELECT *
FROM stanowisko_zrodla_wpisy
ORDER BY ocena = 'nierzetelne' DESC, ocena = 'zastrzezenia' DESC, nazwa, id;

-- name: wpisz_zrodlo :exec
-- Nowe źródło, jeszcze nieocenione; identyfikator (UUID) nadaje aplikacja, żeby ustalenie mogło go
-- od razu wskazać. Uczestnika sprawy wskazuje odnośnik zamiast nazwy.
INSERT INTO stanowisko_zrodla_wpisy (id, nazwa, rodzaj, uczestnik_odnosnik, tozsamosc, ocena)
VALUES (:id, :nazwa, :rodzaj, :uczestnik_odnosnik, :tozsamosc, 'nieoceniona');

-- name: ocen_zrodlo :exec
-- Nowa ocena źródła; obniżyć ją może tylko zdarzenie z faktem.
-- param ocena: "rzetelne" | "zastrzezenia" | "nierzetelne"
-- param oceniono: string
UPDATE stanowisko_zrodla_wpisy
SET ocena = :ocena,
  oceniono = :oceniono
WHERE id = :id;

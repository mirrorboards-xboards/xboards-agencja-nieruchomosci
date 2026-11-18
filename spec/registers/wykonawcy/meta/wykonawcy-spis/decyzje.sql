-- name: decyzje_wykonawcow :many
-- Decyzje człowieka przepisane do rejestru, najnowsze pierwsze.
SELECT *
FROM wykonawcy_spis_decyzje
ORDER BY podpisano DESC, id;

-- name: wpisz_decyzje_o_wykonawcy :exec
-- Decyzja człowieka podpisana jego kluczem, przepisana do rejestru razem z podpisem, zanim
-- Kierownik najmu wpisze zmianę, której dotyczy. Identyfikator (UUID) nadaje aplikacja.
INSERT INTO wykonawcy_spis_decyzje (id, rodzaj, decyduje, tresc, dotyczy_skrot, podpis, podpisano,
  wpisal, wpisano)
VALUES (:id, :rodzaj, :decyduje, :tresc, :dotyczy_skrot, :podpis, :podpisano, :wpisal, :wpisano);

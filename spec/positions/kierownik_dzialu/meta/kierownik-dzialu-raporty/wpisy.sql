-- name: raporty :many
-- Raporty agentów działu, najnowsze pierwsze.
SELECT *
FROM kierownik_dzialu_raporty_wpisy
ORDER BY otrzymano DESC, id;

-- name: raporty_agenta :many
-- Raporty jednego agenta, od najstarszego.
SELECT *
FROM kierownik_dzialu_raporty_wpisy
WHERE agent_id = :agent_id
ORDER BY otrzymano, id;

-- name: przyjmij_raport :exec
-- Raport agenta o stałych polach; identyfikator (UUID) nadaje aplikacja, żeby prośba, zajrzenie
-- i rozesłanie polisy mogły go od razu wskazać.
INSERT INTO kierownik_dzialu_raporty_wpisy (id, agent_id, otrzymano, rodzaj, etap_albo_okres, blokada,
  najblizszy_termin, decyzja_odnosnik, uwagi)
VALUES (:id, :agent_id, :otrzymano, :rodzaj, :etap_albo_okres, :blokada, :najblizszy_termin,
  :decyzja_odnosnik, :uwagi);

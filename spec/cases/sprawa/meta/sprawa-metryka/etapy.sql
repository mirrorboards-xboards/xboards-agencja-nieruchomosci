-- name: etapy :many
-- Dziennik etapów sprawy w kolejności wejścia; ostatni wiersz to bieżący etap.
SELECT *
FROM sprawa_metryka_etapy
ORDER BY id;

-- name: biezacy_etap :one
-- Etap, w którym sprawa jest teraz, z datą wejścia.
SELECT *
FROM sprawa_metryka_etapy
ORDER BY id DESC
LIMIT 1;

-- name: zmien_etap_sprawy :exec
-- Wejście sprawy w kolejny etap; aplikacja budzi wtedy kierownika działu. Przerwanie wskazuje
-- zatwierdzoną decyzję przerwanie_sprawy, a powrót z przerwania do etapu sprzed niego decyzję
-- wznowienie_sprawy. Zamknięcie przychodzi po raporcie końcowym.
INSERT INTO sprawa_metryka_etapy (etap, od, zmienil, decyzja_id)
VALUES (:etap, :od, :zmienil, :decyzja_id);

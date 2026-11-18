-- name: wpuszczenia_otwarte :many
-- Agenci, którzy są teraz w pokojach działu, według pokoju.
SELECT *
FROM kierownik_dzialu_agenci_wpuszczenia
WHERE usunieto IS NULL
ORDER BY pokoj_odnosnik, wpuszczono, id;

-- name: wpuszczenia_pokoju :many
-- Wszystkie wpuszczenia do jednego pokoju, także zamknięte, od najstarszego.
SELECT *
FROM kierownik_dzialu_agenci_wpuszczenia
WHERE pokoj_odnosnik = :pokoj_odnosnik
ORDER BY wpuszczono, id;

-- name: odnotuj_wpuszczenie :exec
-- Agent wszedł do teczki albo pokoju wymiany: własny agent kierownika (agent_id) albo agent innego
-- działu na prośbę jego kierownika (prosil, prosba_odnosnik).
INSERT INTO kierownik_dzialu_agenci_wpuszczenia (tozsamosc, stanowisko, agent_id, pokoj_odnosnik,
  klient_odnosnik, prosil, prosba_odnosnik, wpuscil, wpuszczono, zatwierdzil, podpis, zatwierdzono)
VALUES (:tozsamosc, :stanowisko, :agent_id, :pokoj_odnosnik, :klient_odnosnik, :prosil,
  :prosba_odnosnik, :wpuscil, :wpuszczono, :zatwierdzil, :podpis, :zatwierdzono);

-- name: zatwierdz_wpuszczenie :exec
-- Człowiek zatwierdził wpuszczenie podpisaną wiadomością, gdy przyszła po wpuszczeniu.
-- param zatwierdzil: string
-- param podpis: string
-- param zatwierdzono: string
UPDATE kierownik_dzialu_agenci_wpuszczenia
SET zatwierdzil = :zatwierdzil,
  podpis = :podpis,
  zatwierdzono = :zatwierdzono
WHERE id = :id;

-- name: odnotuj_usuniecie_z_pokoju :exec
-- Agent wyszedł z pokoju, np. bo jego praca się skończyła albo kierownik go wymienił.
-- param usunieto: string
UPDATE kierownik_dzialu_agenci_wpuszczenia
SET usunieto = :usunieto
WHERE id = :id;

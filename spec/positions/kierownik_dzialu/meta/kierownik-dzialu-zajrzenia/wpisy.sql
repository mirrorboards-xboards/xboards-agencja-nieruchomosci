-- name: zajrzenia :many
-- Wszystkie zajrzenia kierownika do teczek, najnowsze pierwsze.
SELECT *
FROM kierownik_dzialu_zajrzenia_wpisy
ORDER BY zajrzano DESC, id;

-- name: zajrzenia_od :many
-- Zajrzenia od podanej chwili, np. do raportu dla właściciela.
SELECT *
FROM kierownik_dzialu_zajrzenia_wpisy
WHERE zajrzano >= :zajrzano
ORDER BY zajrzano, id;

-- name: zapisz_zajrzenie :exec
-- Zajrzenie do teczki z powodem i opisem; raport, jeśli to on sygnalizował problem.
INSERT INTO kierownik_dzialu_zajrzenia_wpisy (teczka_odnosnik, zajrzano, powod, opis, raport_id)
VALUES (:teczka_odnosnik, :zajrzano, :powod, :opis, :raport_id);

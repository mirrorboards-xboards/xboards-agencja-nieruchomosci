-- name: decyzje_kontroli :many
-- Prośby i rozstrzygnięcia, najnowsze pierwsze.
SELECT *
FROM rejestr_kontroli_decyzje_wpisy
ORDER BY zlozono DESC, id;

-- name: decyzje_kontroli_czekajace :many
-- Prośby, które czekają na rozstrzygnięcie człowieka, najstarsze pierwsze.
SELECT *
FROM rejestr_kontroli_decyzje_wpisy
WHERE stan = 'czeka'
ORDER BY zlozono, id;

-- name: popros_o_decyzje_kontroli :exec
-- Prośba Kierownika kontroli o rozstrzygnięcie. Identyfikator (UUID) nadaje aplikacja; wiersz,
-- którego decyzja dotyczy, może mieć już identyfikator, choć jeszcze go nie ma w rejestrze.
INSERT INTO rejestr_kontroli_decyzje_wpisy (id, rodzaj, prosi, decyduje, decyduje_funkcja, zlozono,
  tresc, dotyczy_tabela, dotyczy_id, dotyczy_skrot, stan)
VALUES (:id, :rodzaj, :prosi, :decyduje, :decyduje_funkcja, :zlozono, :tresc, :dotyczy_tabela,
  :dotyczy_id, :dotyczy_skrot, 'czeka');

-- name: rozstrzygnij_decyzje_kontroli :exec
-- Rozstrzygnięcie człowieka z podpisem jego kluczem; zdjęcie wątpliwości ma uzasadnienie.
UPDATE rejestr_kontroli_decyzje_wpisy
SET stan = 'rozstrzygnieta',
  rozstrzygniecie = :rozstrzygniecie,
  uzasadnienie = :uzasadnienie,
  podpisal = :podpisal,
  podpis = :podpis,
  podpisano = :podpisano,
  teczka_decyzja_odnosnik = :teczka_decyzja_odnosnik
WHERE id = :id;

-- name: uznaj_prosbe_kontroli_za_zbedna :exec
-- Prośba, której już nie trzeba rozstrzygać, z powodem.
-- param powod_zbednosci: string
UPDATE rejestr_kontroli_decyzje_wpisy
SET stan = 'zbedna',
  powod_zbednosci = :powod_zbednosci
WHERE id = :id;

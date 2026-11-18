-- name: pytania_gosci :many
-- Aktualne odpowiedzi z liczbą wystąpień i liczbą tych, w których pomogły; najczęstsze pierwsze.
-- column wystapienia: number
-- column pomocne: number
SELECT pytanie.*,
  count(wystapienie.id) AS wystapienia,
  coalesce(sum(wystapienie.pomogla), 0) AS pomocne
FROM konsjerz_pytania_wpisy AS pytanie
LEFT JOIN konsjerz_pytania_wystapienia AS wystapienie
  ON wystapienie.pytanie_id = pytanie.id
WHERE pytanie.zastapione_przez_id IS NULL
GROUP BY pytanie.id
ORDER BY wystapienia DESC, pytanie.temat, pytanie.id;

-- name: zapisz_pytanie :exec
-- Nowe pytanie z odpowiedzią, bez danych gościa.
INSERT INTO konsjerz_pytania_wpisy (temat, odpowiedz, zapisano)
VALUES (:temat, :odpowiedz, :zapisano);

-- name: zastap_odpowiedz :exec
-- Poprzednia odpowiedź staje się nieaktualna, bo zastępuje ją lepsza na ten sam temat.
-- param zastapione_przez_id: number
UPDATE konsjerz_pytania_wpisy
SET zastapione_przez_id = :zastapione_przez_id
WHERE id = :id;

-- name: odnotuj_propozycje_opisu :exec
-- Konsjerż zaproponował Zarządcy zmianę opisu lokalu notatką w teczce lokalu.
-- param propozycja_przekazano: string
-- param notatka_odnosnik: string
UPDATE konsjerz_pytania_wpisy
SET propozycja_przekazano = :propozycja_przekazano,
  notatka_odnosnik = :notatka_odnosnik
WHERE id = :id;

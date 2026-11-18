-- name: sprawozdania :many
-- Wszystkie sprawozdania, najnowsze pierwsze; robocze mają puste przekazanie.
SELECT *
FROM kierownik_biura_sprawozdania_wpisy
ORDER BY sporzadzono DESC, id;

-- name: zapisz_sprawozdanie :exec
-- Nowe sprawozdanie robocze.
INSERT INTO kierownik_biura_sprawozdania_wpisy (rodzaj, okres_od, okres_do, sciezka, skrot,
  sporzadzono)
VALUES (:rodzaj, :okres_od, :okres_do, :sciezka, :skrot, :sporzadzono);

-- name: zmien_sprawozdanie :exec
-- Nowa wersja roboczego sprawozdania w XGit.
UPDATE kierownik_biura_sprawozdania_wpisy
SET okres_od = :okres_od,
  okres_do = :okres_do,
  sciezka = :sciezka,
  skrot = :skrot,
  sporzadzono = :sporzadzono
WHERE id = :id;

-- name: przekaz_sprawozdanie :exec
-- Sprawozdanie przekazane właścicielowi biura.
-- param przekazano: string
UPDATE kierownik_biura_sprawozdania_wpisy
SET przekazano = :przekazano
WHERE id = :id;

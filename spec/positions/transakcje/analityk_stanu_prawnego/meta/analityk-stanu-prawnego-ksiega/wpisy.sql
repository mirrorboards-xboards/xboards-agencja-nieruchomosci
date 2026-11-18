-- name: wpisy_stanu_ksiegi :many
-- Wpisy, wzmianki i ostrzeżenia jednego stanu księgi według działów.
SELECT *
FROM analityk_stanu_prawnego_ksiega_wpisy
WHERE stan_id = :stan_id
ORDER BY dzial, numer, rodzaj, id;

-- name: zapisz_wpis_ksiegi :exec
-- Wpis, wzmianka albo ostrzeżenie z ostatniego stanu księgi, bez danych osób.
INSERT INTO analityk_stanu_prawnego_ksiega_wpisy (stan_id, dzial, numer, rodzaj, przedmiot)
VALUES (:stan_id, :dzial, :numer, :rodzaj, :przedmiot);

-- name: zmiany_ksiegi :many
-- Co się zmieniło w księdze między dwoma ostatnimi stanami: wpis, którego wcześniej nie było
-- (dodany), i wpis, którego już nie ma (usuniety). Zmieniona treść wpisu o tym samym numerze to
-- jeden usunięty i jeden dodany. Przy jednym stanie nie ma z czym porównać, więc zmian nie ma.
-- column zmiana: "dodany" | "usuniety"
WITH ostatnie AS (
  SELECT id,
    chwila
  FROM analityk_stanu_prawnego_ksiega_stany
  WHERE numer_kw = :numer_kw
  ORDER BY chwila DESC
  LIMIT 2
),
nowy AS (
  SELECT id
  FROM ostatnie
  ORDER BY chwila DESC
  LIMIT 1
),
stary AS (
  SELECT id
  FROM ostatnie
  ORDER BY chwila DESC
  LIMIT 1 OFFSET 1
)
SELECT 'dodany' AS zmiana,
  wpis.dzial,
  wpis.numer,
  wpis.rodzaj,
  wpis.przedmiot
FROM analityk_stanu_prawnego_ksiega_wpisy AS wpis
WHERE wpis.stan_id = (SELECT id FROM nowy)
  AND EXISTS (SELECT 1 FROM stary)
  AND NOT EXISTS (
    SELECT 1
    FROM analityk_stanu_prawnego_ksiega_wpisy AS dawny
    WHERE dawny.stan_id = (SELECT id FROM stary)
      AND dawny.dzial = wpis.dzial
      AND dawny.numer = wpis.numer
      AND dawny.rodzaj = wpis.rodzaj
      AND dawny.przedmiot = wpis.przedmiot
  )
UNION ALL
SELECT 'usuniety' AS zmiana,
  wpis.dzial,
  wpis.numer,
  wpis.rodzaj,
  wpis.przedmiot
FROM analityk_stanu_prawnego_ksiega_wpisy AS wpis
WHERE wpis.stan_id = (SELECT id FROM stary)
  AND NOT EXISTS (
    SELECT 1
    FROM analityk_stanu_prawnego_ksiega_wpisy AS obecny
    WHERE obecny.stan_id = (SELECT id FROM nowy)
      AND obecny.dzial = wpis.dzial
      AND obecny.numer = wpis.numer
      AND obecny.rodzaj = wpis.rodzaj
      AND obecny.przedmiot = wpis.przedmiot
  )
ORDER BY dzial, numer, zmiana;

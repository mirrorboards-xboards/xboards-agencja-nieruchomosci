-- name: dane_kredytowe :many
-- Dane wejściowe analizy kredytowej w kolejności wpisania.
SELECT *
FROM poszukiwanie_kredyt_dane
ORDER BY wpisano, id;

-- name: wpisz_dana_kredytowa :exec
-- Dana wejściowa z dokumentem albo deklaracja klienta oznaczona jako niepotwierdzona;
-- identyfikator (UUID) nadaje aplikacja, bo wskazują go analizy.
INSERT INTO poszukiwanie_kredyt_dane (id, rodzaj, opis, wartosc, okres_zrodla, dokument_id,
  deklaracja, wpisano)
VALUES (:id, :rodzaj, :opis, :wartosc, :okres_zrodla, :dokument_id, :deklaracja, :wpisano);

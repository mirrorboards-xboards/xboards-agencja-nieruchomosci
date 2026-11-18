-- name: informacje_klienta :many
-- Kiedy i jak biuro poinformowało klienta o danych w kartotece.
SELECT *
FROM klienci_kartoteka_informacje
WHERE klient_id = :klient_id
ORDER BY poinformowano, id;

-- name: wpisz_informacje_klienta :exec
-- Informacja o kartotece wysłana klientowi przez agenta sprawy (art. 13 RODO).
INSERT INTO klienci_kartoteka_informacje (klient_id, poinformowano, sposob, wysylka_odnosnik)
VALUES (:klient_id, :poinformowano, :sposob, :wysylka_odnosnik);

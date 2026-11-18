-- name: informacje_aml_klienta :many
-- Informacje dla klienta o przetwarzaniu danych i obowiązkach biura.
SELECT *
FROM rejestr_kontroli_sprawdzenia_informacje
WHERE klient_id = :klient_id
ORDER BY id DESC;

-- name: wpisz_informacje_aml :exec
-- Kiedy i jak biuro poinformowało klienta przed nawiązaniem stosunków (art. 34 ust. 5 AML).
INSERT INTO rejestr_kontroli_sprawdzenia_informacje (klient_id, poinformowano, sposob, zrodlo,
  raport_odnosnik, wpisal, wpisano)
VALUES (:klient_id, :poinformowano, :sposob, :zrodlo, :raport_odnosnik, :wpisal, :wpisano);

-- name: adresy_klienta :many
-- Adresy klienta, bieżące i zakończone, najnowsze pierwsze.
SELECT *
FROM klienci_kartoteka_adresy
WHERE klient_id = :klient_id
ORDER BY id DESC;

-- name: wpisz_adres_klienta :exec
-- Nowy adres klienta ze źródłem w teczce. Bieżący adres tego samego rodzaju kończy się w dniu,
-- od którego obowiązuje nowy.
UPDATE klienci_kartoteka_adresy
SET do = :od
WHERE klient_id = :klient_id
  AND rodzaj = :rodzaj
  AND do IS NULL;
INSERT INTO klienci_kartoteka_adresy (klient_id, rodzaj, adres, od, zrodlo_odnosnik)
VALUES (:klient_id, :rodzaj, :adres, :od, :zrodlo_odnosnik);

-- name: zakoncz_adres_klienta :exec
-- Adres, który przestał obowiązywać bez następcy, np. adres do korespondencji, gdy klient chce
-- dostawać pisma na adres zamieszkania.
UPDATE klienci_kartoteka_adresy
SET do = :do
WHERE id = :id;

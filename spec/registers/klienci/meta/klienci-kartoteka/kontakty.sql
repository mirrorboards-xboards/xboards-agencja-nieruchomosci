-- name: kontakty_klienta :many
-- Kontakty klienta, bieżące i zakończone, najnowsze pierwsze.
SELECT *
FROM klienci_kartoteka_kontakty
WHERE klient_id = :klient_id
ORDER BY id DESC;

-- name: klienci_po_kontakcie :many
-- Klienci z tym telefonem albo adresem poczty, także dawnym: stary numer z karty Recepcji nadal
-- wskazuje klienta.
SELECT *
FROM klienci_kartoteka_kontakty
WHERE wartosc = :wartosc
ORDER BY klient_id, id;

-- name: wpisz_kontakt_klienta :exec
-- Nowy telefon albo adres poczty klienta ze źródłem w teczce. Wiadomość „zmieniłam numer” nie
-- jest źródłem; agent sprawy potwierdza zmianę innym kanałem.
INSERT INTO klienci_kartoteka_kontakty (klient_id, rodzaj, wartosc, preferowany, od, zrodlo_odnosnik)
VALUES (:klient_id, :rodzaj, :wartosc, :preferowany, :od, :zrodlo_odnosnik);

-- name: zakoncz_kontakt_klienta :exec
-- Kontakt, którego klient już nie używa; zostaje w kartotece jako historia.
UPDATE klienci_kartoteka_kontakty
SET do = :do
WHERE id = :id;

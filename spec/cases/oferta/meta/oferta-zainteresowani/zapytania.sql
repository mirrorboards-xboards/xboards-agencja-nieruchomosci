-- name: zapytania :many
-- Zapytania o ofertę w kolejności wpisania.
SELECT *
FROM oferta_zainteresowani_zapytania
ORDER BY id;

-- name: wpisz_zapytanie :exec
-- Zapytanie zainteresowanego z wpływem, w którym przyszło.
INSERT INTO oferta_zainteresowani_zapytania (osoba_id, wplyw_id)
VALUES (:osoba_id, :wplyw_id);

-- name: odnotuj_odpowiedz_na_zapytanie :exec
-- Wysyłka z odpowiedzią na zapytanie; zainteresowany ma już informację o swoich danych.
-- param odpowiedz_wysylka_id: string
UPDATE oferta_zainteresowani_zapytania
SET odpowiedz_wysylka_id = :odpowiedz_wysylka_id
WHERE id = :id;

-- name: odnotuj_wynik_zapytania :exec
-- Czy zapytanie skończyło się prezentacją.
-- param skonczylo_sie_prezentacja: boolean
UPDATE oferta_zainteresowani_zapytania
SET skonczylo_sie_prezentacja = :skonczylo_sie_prezentacja
WHERE id = :id;

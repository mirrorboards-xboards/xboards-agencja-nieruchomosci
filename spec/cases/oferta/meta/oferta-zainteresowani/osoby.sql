-- name: zainteresowani :many
-- Osoby, które pytały o ofertę albo ją oglądały, w kolejności pozyskania danych.
SELECT *
FROM oferta_zainteresowani_osoby
ORDER BY dane_pozyskano, id;

-- name: wpisz_zainteresowanego :exec
-- Osoba, która pyta albo ogląda: imię, kontakt i to, czy działa sama, przez inne biuro albo jako
-- klient biura. Identyfikator (UUID) nadaje aplikacja, bo wskazują go zapytania, prezentacje
-- i oferty.
INSERT INTO oferta_zainteresowani_osoby (id, imie, nazwisko, telefon, email, dziala_jako, biuro,
  dla_kogo, dane_pozyskano, informacja_wysylka_id, informacja_od_recepcji)
VALUES (:id, :imie, :nazwisko, :telefon, :email, :dziala_jako, :biuro, :dla_kogo, :dane_pozyskano,
  :informacja_wysylka_id, :informacja_od_recepcji);

-- name: zmien_kontakt_zainteresowanego :exec
-- Poprawiony kontakt osoby, np. nowy numer telefonu.
UPDATE oferta_zainteresowani_osoby
SET telefon = :telefon,
  email = :email
WHERE id = :id;

-- name: odnotuj_informacje_dla_zainteresowanego :exec
-- Informacja o przetwarzaniu danych: wysłana wysyłka z informacja_rodo albo dzień, w którym osoba
-- dostała ją od Recepcji. Wpisuje się ją raz.
UPDATE oferta_zainteresowani_osoby
SET informacja_wysylka_id = :informacja_wysylka_id,
  informacja_od_recepcji = :informacja_od_recepcji
WHERE id = :id;

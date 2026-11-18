-- name: wersje_profili :many
-- Wersje profili od najstarszej, także niepotwierdzone.
SELECT *
FROM poszukiwanie_profile_wersje
ORDER BY profil_id, od;

-- name: profile_biezace :many
-- Najnowsza potwierdzona wersja każdego profilu; z nią porównuje się propozycje.
SELECT *
FROM poszukiwanie_profile_biezace
ORDER BY profil_id;

-- name: cechy_profili :many
-- Cechy wersji profili z oznaczeniem warunku albo życzenia.
SELECT *
FROM poszukiwanie_profile_cechy
ORDER BY wersja_id, id;

-- name: finansowanie_profili :many
-- Źródła pieniędzy wersji profili według deklaracji klienta.
SELECT *
FROM poszukiwanie_profile_finansowanie
ORDER BY wersja_id, id;

-- name: wpisz_wersje_profilu :exec
-- Nowa, jeszcze niepotwierdzona wersja profilu; identyfikator (UUID) nadaje aplikacja, bo
-- wskazują go cechy, finansowanie, zgoda klienta i propozycje.
INSERT INTO poszukiwanie_profile_wersje (id, profil_id, cel, na_wynajem, rodzaj_nieruchomosci,
  przeznaczenie, polozenie, dojazd, pow_min, pow_max, pokoje_min, pokoje_max, pietro, termin,
  termin_powod, budzet, budzet_z_kosztami, najem_od, okres, osoby, dzieci, zwierzeta,
  okazjonalny_dopuszcza, inny_lokal_ma, kaucja_max, umeblowany, od)
VALUES (:id, :profil_id, :cel, :na_wynajem, :rodzaj_nieruchomosci, :przeznaczenie, :polozenie,
  :dojazd, :pow_min, :pow_max, :pokoje_min, :pokoje_max, :pietro, :termin, :termin_powod, :budzet,
  :budzet_z_kosztami, :najem_od, :okres, :osoby, :dzieci, :zwierzeta, :okazjonalny_dopuszcza,
  :inny_lokal_ma, :kaucja_max, :umeblowany, :od);

-- name: dopisz_ceche_profilu :exec
-- Cecha niepotwierdzonej wersji: warunek albo życzenie.
INSERT INTO poszukiwanie_profile_cechy (wersja_id, cecha, waga)
VALUES (:wersja_id, :cecha, :waga);

-- name: dopisz_finansowanie_profilu :exec
-- Źródło pieniędzy niepotwierdzonej wersji z kwotą według deklaracji klienta.
INSERT INTO poszukiwanie_profile_finansowanie (wersja_id, zrodlo, kwota)
VALUES (:wersja_id, :zrodlo, :kwota);

-- name: potwierdz_wersje_profilu :exec
-- Klient potwierdził wersję profilu zgodą potwierdzenie_profilu; od tej chwili wersja się nie
-- zmienia.
-- param zgoda_id: string
UPDATE poszukiwanie_profile_wersje
SET zgoda_id = :zgoda_id
WHERE id = :id;

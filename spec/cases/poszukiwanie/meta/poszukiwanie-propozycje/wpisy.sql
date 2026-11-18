-- name: propozycje :many
-- Propozycje ofert dla klienta, najnowsze pierwsze.
SELECT *
FROM poszukiwanie_propozycje_wpisy
ORDER BY utworzono DESC, id;

-- name: wpisz_propozycje :exec
-- Oferta z portalu, z katalogu ofert biura albo od innego biura, porównana z bieżącą potwierdzoną
-- wersją profilu; identyfikator (UUID) nadaje aplikacja, bo wskazują go prezentacje i oferty
-- klienta.
INSERT INTO poszukiwanie_propozycje_wpisy (id, profil_wersja_id, zrodlo, ogloszenie_sciezka,
  ogloszenie_skrot, portal, numer, katalog_odnosnik, katalog_wersja_od, prowadzi, niepubliczna,
  zgodnosc, niespelnione_zyczenia, rozbieznosci, numer_kw, cena_ogloszenia, ceny_rcn_dokument_id,
  utworzono)
VALUES (:id, :profil_wersja_id, :zrodlo, :ogloszenie_sciezka, :ogloszenie_skrot, :portal, :numer,
  :katalog_odnosnik, :katalog_wersja_od, :prowadzi, :niepubliczna, :zgodnosc,
  :niespelnione_zyczenia, :rozbieznosci, :numer_kw, :cena_ogloszenia, :ceny_rcn_dokument_id,
  :utworzono);

-- name: odnotuj_przegladanie_ksiegi_propozycji :exec
-- Wynik przeglądania księgi oglądanej nieruchomości, bez oceny skutków; wpisuje się go raz.
-- param numer_kw: string
-- param kw_przegladano: string
-- param kw_ogloszajacy_wpisany: "tak" | "nie" | "nieznane"
UPDATE poszukiwanie_propozycje_wpisy
SET numer_kw = :numer_kw,
  kw_przegladano = :kw_przegladano,
  kw_ogloszajacy_wpisany = :kw_ogloszajacy_wpisany,
  kw_wpisy_iii_iv = :kw_wpisy_iii_iv
WHERE id = :id;

-- name: odnotuj_ceny_transakcji_propozycji :exec
-- Dokument z cenami podobnych transakcji z rejestru cen nieruchomości, który Opiekun pokazał
-- klientowi.
-- param ceny_rcn_dokument_id: string
UPDATE poszukiwanie_propozycje_wpisy
SET ceny_rcn_dokument_id = :ceny_rcn_dokument_id
WHERE id = :id;

-- name: odnotuj_rozbieznosci_propozycji :exec
-- Rozbieżności zauważone jako fakt, bez oceny; wpisuje się je raz.
-- param rozbieznosci: string
UPDATE poszukiwanie_propozycje_wpisy
SET rozbieznosci = :rozbieznosci
WHERE id = :id;

-- name: odnotuj_przedstawienie_propozycji :exec
-- Chwila, w której klient dostał propozycję; nie wcześniej niż praca według umowy.
-- param przedstawiono: string
UPDATE poszukiwanie_propozycje_wpisy
SET przedstawiono = :przedstawiono
WHERE id = :id;

-- name: odnotuj_reakcje_na_propozycje :exec
-- Reakcja klienta z powodem; po „czeka” klient może odpowiedzieć jeszcze raz.
-- param reakcja: "chce_obejrzec" | "odrzuca" | "czeka"
UPDATE poszukiwanie_propozycje_wpisy
SET reakcja = :reakcja,
  reakcja_powod = :reakcja_powod
WHERE id = :id;

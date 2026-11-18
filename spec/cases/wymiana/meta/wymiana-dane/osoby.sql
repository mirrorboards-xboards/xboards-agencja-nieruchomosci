-- name: dane_osob :many
-- Dane osób do aktu z wpisami, które je przyniosły; poprawione zostają obok poprawek.
SELECT osoba.*, wpis.autor, wpis.chwila, wpis.poprawia_id
FROM wymiana_dane_osoby AS osoba
JOIN wymiana_wpisy_wpisy AS wpis ON wpis.id = osoba.wpis_id
ORDER BY wpis.rowid;

-- name: przekaz_dane_osoby :exec
-- Biuro przekazuje w imieniu strony dane osoby, która bierze udział w akcie, ze źródłem w teczce.
INSERT INTO wymiana_wpisy_wpisy (id, strona, autor, rodzaj, chwila, odpowiada_na_id, poprawia_id)
VALUES (:wpis_id, :strona, :autor, 'dane_osoby', :chwila, :odpowiada_na_id, :poprawia_id);

INSERT INTO wymiana_dane_osoby (wpis_id, strona_id, udzial, rodzaj_podmiotu, imiona, nazwisko,
  imiona_rodzicow, miejsce_zamieszkania, nazwa, siedziba, obywatelstwo, stan_cywilny,
  zrodlo_dokument_odnosnik)
VALUES (:wpis_id, :strona_id, :udzial, :rodzaj_podmiotu, :imiona, :nazwisko, :imiona_rodzicow,
  :miejsce_zamieszkania, :nazwa, :siedziba, :obywatelstwo, :stan_cywilny,
  :zrodlo_dokument_odnosnik);

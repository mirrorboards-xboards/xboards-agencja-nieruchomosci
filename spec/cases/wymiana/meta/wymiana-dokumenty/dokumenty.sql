-- name: dokumenty_pokoju :many
-- Dokumenty, pełnomocnictwa, zgody, wersje projektu i koszty z wpisami, które je przyniosły.
SELECT dokument.*, wpis.strona, wpis.autor, wpis.chwila, wpis.poprawia_id, wpis.tresc,
  wpis.sciezka, wpis.skrot
FROM wymiana_dokumenty_wpisy AS dokument
JOIN wymiana_wpisy_wpisy AS wpis ON wpis.id = dokument.wpis_id
ORDER BY wpis.rowid;

-- name: wersje_projektu_aktu :many
-- Wersje projektu aktu od najnowszej, z plikiem i tym, czy kancelaria wskazała ją jako ostateczną.
SELECT dokument.wpis_id, dokument.numer_wersji, dokument.ostateczna, wpis.chwila, wpis.tresc,
  wpis.sciezka, wpis.skrot
FROM wymiana_dokumenty_wpisy AS dokument
JOIN wymiana_wpisy_wpisy AS wpis ON wpis.id = dokument.wpis_id
WHERE dokument.rodzaj = 'projekt_aktu'
ORDER BY dokument.numer_wersji DESC;

-- name: przekaz_dokument_z_listy :exec
-- Biuro przekazuje w imieniu strony dokument z listy kancelarii: wystawca, data, ważność, plik
-- i to, czy oryginał będzie w dniu aktu.
-- param sciezka: string
-- param skrot: string
INSERT INTO wymiana_wpisy_wpisy (id, strona, autor, rodzaj, chwila, odpowiada_na_id, poprawia_id,
  tresc, sciezka, skrot)
VALUES (:wpis_id, :strona, :autor, 'dokument', :chwila, :odpowiada_na_id, :poprawia_id, :tresc,
  :sciezka, :skrot);

INSERT INTO wymiana_dokumenty_wpisy (wpis_id, rodzaj, pozycja_id, strona_id, wystawca,
  data_wystawienia, waznosc_do, oryginal_w_dniu_aktu, kto_przyniesie)
VALUES (:wpis_id, 'dokument', :pozycja_id, :strona_id, :wystawca, :data_wystawienia, :waznosc_do,
  :oryginal_w_dniu_aktu, :kto_przyniesie);

-- name: przekaz_pelnomocnictwo :exec
-- Biuro przekazuje pełnomocnictwo strony z zakresem; dopiero po nim pisze w jej imieniu.
-- param sciezka: string
-- param skrot: string
INSERT INTO wymiana_wpisy_wpisy (id, strona, autor, rodzaj, chwila, poprawia_id, tresc, sciezka,
  skrot)
VALUES (:wpis_id, :strona, :autor, 'pelnomocnictwo', :chwila, :poprawia_id, :tresc, :sciezka,
  :skrot);

INSERT INTO wymiana_dokumenty_wpisy (wpis_id, rodzaj, strona_id, wystawca, data_wystawienia,
  waznosc_do, zakres)
VALUES (:wpis_id, 'pelnomocnictwo', :strona_id, :wystawca, :data_wystawienia, :waznosc_do, :zakres);

-- name: przekaz_zgode_stron :exec
-- Biuro przekazuje zgodę strony, np. na to, żeby biuro dostawało projekt aktu i informację po akcie.
-- param sciezka: string
-- param skrot: string
INSERT INTO wymiana_wpisy_wpisy (id, strona, autor, rodzaj, chwila, poprawia_id, tresc, sciezka,
  skrot)
VALUES (:wpis_id, :strona, :autor, 'zgoda_stron', :chwila, :poprawia_id, :tresc, :sciezka, :skrot);

INSERT INTO wymiana_dokumenty_wpisy (wpis_id, rodzaj, strona_id, data_wystawienia, zakres)
VALUES (:wpis_id, 'zgoda_stron', :strona_id, :data_wystawienia, :zakres);

-- name: przekaz_projekt_aktu :exec
-- Kancelaria zostawia kolejną wersję projektu aktu, za zgodą stron na jego przekazanie biuru.
-- param sciezka: string
-- param skrot: string
INSERT INTO wymiana_wpisy_wpisy (id, strona, autor, rodzaj, chwila, odpowiada_na_id, tresc,
  sciezka, skrot)
VALUES (:wpis_id, :strona, :autor, 'projekt_aktu', :chwila, :odpowiada_na_id, :tresc, :sciezka,
  :skrot);

INSERT INTO wymiana_dokumenty_wpisy (wpis_id, rodzaj, numer_wersji, ostateczna)
VALUES (:wpis_id, 'projekt_aktu', :numer_wersji, :ostateczna);

-- name: podaj_koszty_aktu :exec
-- Kancelaria podaje taksę, opłaty i podatki, które pobierze; biuro przekazuje je stronom.
INSERT INTO wymiana_wpisy_wpisy (id, strona, autor, rodzaj, chwila, poprawia_id, tresc, sciezka,
  skrot)
VALUES (:wpis_id, :strona, :autor, 'koszty', :chwila, :poprawia_id, :tresc, :sciezka, :skrot);

INSERT INTO wymiana_dokumenty_wpisy (wpis_id, rodzaj)
VALUES (:wpis_id, 'koszty');

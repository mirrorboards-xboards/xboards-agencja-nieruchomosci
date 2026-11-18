-- name: uwagi_do_projektu :many
-- Uwagi stron do wersji projektu, z decyzją, która pozwoliła je przekazać.
SELECT uwagi.*, projekt.numer_wersji, wpis.chwila, wpis.tresc, wpis.sciezka, wpis.skrot
FROM wymiana_dokumenty_uwagi AS uwagi
JOIN wymiana_dokumenty_wpisy AS projekt ON projekt.wpis_id = uwagi.projekt_id
JOIN wymiana_wpisy_wpisy AS wpis ON wpis.id = uwagi.wpis_id
ORDER BY wpis.rowid;

-- name: przekaz_uwagi_do_projektu :exec
-- Biuro przekazuje uwagi stron do wskazanej wersji projektu za zgodą człowieka z teczki.
INSERT INTO wymiana_wpisy_wpisy (id, strona, autor, rodzaj, chwila, odpowiada_na_id, tresc,
  sciezka, skrot)
VALUES (:wpis_id, :strona, :autor, 'uwagi_do_projektu', :chwila, :projekt_id, :tresc, :sciezka,
  :skrot);

INSERT INTO wymiana_dokumenty_uwagi (wpis_id, projekt_id, decyzja_odnosnik)
VALUES (:wpis_id, :projekt_id, :decyzja_odnosnik);

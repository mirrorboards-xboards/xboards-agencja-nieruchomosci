-- name: dane_nieruchomosci :many
-- Oznaczenie nieruchomości ze źródłem, z wpisami, które je przyniosły.
SELECT nieruchomosc.*, wpis.autor, wpis.chwila, wpis.poprawia_id
FROM wymiana_dane_nieruchomosci AS nieruchomosc
JOIN wymiana_wpisy_wpisy AS wpis ON wpis.id = nieruchomosc.wpis_id
ORDER BY wpis.rowid;

-- name: przekaz_dane_nieruchomosci :exec
-- Biuro przekazuje w imieniu strony oznaczenie nieruchomości z odpisu księgi i z ewidencji.
INSERT INTO wymiana_wpisy_wpisy (id, strona, autor, rodzaj, chwila, odpowiada_na_id, poprawia_id)
VALUES (:wpis_id, :strona, :autor, 'dane_nieruchomosci', :chwila, :odpowiada_na_id, :poprawia_id);

INSERT INTO wymiana_dane_nieruchomosci (wpis_id, strona_id, numer_kw, oznaczenie, dzialki, zrodlo)
VALUES (:wpis_id, :strona_id, :numer_kw, :oznaczenie, :dzialki, :zrodlo);

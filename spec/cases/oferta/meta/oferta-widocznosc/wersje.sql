-- name: wersje_widocznosci :many
-- Wersje rodzaju i widoczności oferty od najstarszej.
SELECT *
FROM oferta_widocznosc_wersje
ORDER BY od, id;

-- name: biezaca_widocznosc :one
-- Rodzaj i widoczność oferty, które obowiązują teraz.
SELECT *
FROM oferta_widocznosc_biezaca;

-- name: wpisz_wersje_widocznosci :exec
-- Rodzaj i widoczność oferty z zawartej umowy (od chwili zawarcia, bez aneksu) albo z aneksu,
-- który je zmienił (od chwili aneksu); identyfikator (UUID) nadaje aplikacja, bo wskazuje go
-- katalog ofert.
INSERT INTO oferta_widocznosc_wersje (id, umowa_id, rodzaj_oferty, widocznosc, granice_zgody,
  aneks_zdarzenie_id, od)
VALUES (:id, :umowa_id, :rodzaj_oferty, :widocznosc, :granice_zgody, :aneks_zdarzenie_id, :od);

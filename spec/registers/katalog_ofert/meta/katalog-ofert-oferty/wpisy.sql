-- name: oferty :many
-- Oferty katalogu z bieżącą wersją i bieżącym statusem, w kolejności wpisania. Opiekun
-- poszukującego proponuje tylko aktywne, a zarezerwowane z informacją o rezerwacji; niepubliczną
-- pokazuje tylko w granicach zgody.
SELECT
  wpisy.id,
  wpisy.numer_oferty,
  wpisy.teczka_odnosnik,
  wpisy.rodzaj_oferty,
  wpisy.rodzaj_nieruchomosci,
  wersje.id AS wersja_id,
  wersje.od AS wersja_od,
  wersje.powierzchnia,
  wersje.lokalizacja_przyblizona,
  wersje.cena_sprzedazy,
  wersje.czynsz,
  wersje.opis,
  wersje.zdjecia_sciezka,
  wersje.widocznosc,
  wersje.granice_zgody,
  wersje.wylacznosc,
  statusy.status,
  statusy.od AS status_od
FROM katalog_ofert_oferty_wpisy AS wpisy
JOIN katalog_ofert_oferty_wersje AS wersje ON wersje.oferta_id = wpisy.id
  AND wersje.id = (
    SELECT max(id)
    FROM katalog_ofert_oferty_wersje
    WHERE oferta_id = wpisy.id
  )
JOIN katalog_ofert_oferty_statusy AS statusy ON statusy.oferta_id = wpisy.id
  AND statusy.id = (
    SELECT max(id)
    FROM katalog_ofert_oferty_statusy
    WHERE oferta_id = wpisy.id
  )
ORDER BY wpisy.rowid;

-- name: oferta_po_numerze :one
-- Oferta o numerze z wiadomości; wpis wskazuje teczkę, której Opiekunowi oferty Recepcja oddaje
-- wiadomość.
SELECT *
FROM katalog_ofert_oferty_wpisy
WHERE numer_oferty = :numer_oferty;

-- name: wpisz_oferte :exec
-- Nowa oferta po raporcie Opiekuna oferty: wpis, pierwsza wersja i status „aktywna” ze źródłem
-- w teczce. Identyfikator (UUID) nadaje aplikacja.
INSERT INTO katalog_ofert_oferty_wpisy (id, numer_oferty, teczka_odnosnik, rodzaj_oferty,
  rodzaj_nieruchomosci, wpisal, wpisano)
VALUES (:id, :numer_oferty, :teczka_odnosnik, :rodzaj_oferty, :rodzaj_nieruchomosci, :wpisal,
  :wpisano);
INSERT INTO katalog_ofert_oferty_wersje (oferta_id, od, powierzchnia, lokalizacja_przyblizona,
  cena_sprzedazy, czynsz, opis, zdjecia_sciezka, zdjecia_skrot, widocznosc, granice_zgody,
  wylacznosc, zrodlo_odnosnik, wpisal)
VALUES (:id, :od, :powierzchnia, :lokalizacja_przyblizona, :cena_sprzedazy, :czynsz, :opis,
  :zdjecia_sciezka, :zdjecia_skrot, :widocznosc, :granice_zgody, :wylacznosc, :zrodlo_odnosnik,
  :wpisal);
INSERT INTO katalog_ofert_oferty_statusy (oferta_id, status, od, zrodlo_odnosnik, wpisal)
VALUES (:id, 'aktywna', :od, :zrodlo_odnosnik, :wpisal);

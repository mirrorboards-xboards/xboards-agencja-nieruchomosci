-- name: wersje_oferty :many
-- Wszystkie wersje oferty, najnowsza pierwsza; Opiekun poszukującego widzi, którą pokazał
-- klientowi.
SELECT *
FROM katalog_ofert_oferty_wersje
WHERE oferta_id = :oferta_id
ORDER BY id DESC;

-- name: wpisz_wersje_oferty :exec
-- Nowa cena, opis, zdjęcia, widoczność albo wyłączność ze źródłem w teczce; zmiana widoczności
-- albo wyłączności ma aneks. Błąd poprzedniej wersji poprawia nowa, która ją wskazuje.
INSERT INTO katalog_ofert_oferty_wersje (oferta_id, od, powierzchnia, lokalizacja_przyblizona,
  cena_sprzedazy, czynsz, opis, zdjecia_sciezka, zdjecia_skrot, widocznosc, granice_zgody,
  wylacznosc, zrodlo_odnosnik, aneks_odnosnik, poprawia_id, wpisal)
VALUES (:oferta_id, :od, :powierzchnia, :lokalizacja_przyblizona, :cena_sprzedazy, :czynsz, :opis,
  :zdjecia_sciezka, :zdjecia_skrot, :widocznosc, :granice_zgody, :wylacznosc, :zrodlo_odnosnik,
  :aneks_odnosnik, :poprawia_id, :wpisal);

-- name: nieruchomosci :many
-- Nieruchomości teczki w kolejności wpisania.
SELECT *
FROM sprawa_nieruchomosci_wpisy
ORDER BY rowid;

-- name: wpisz_nieruchomosc :exec
-- Oznaczenie nieruchomości; identyfikator (UUID) nadaje aplikacja, bo wskazują go dokumenty,
-- umowy i pokój wymiany.
INSERT INTO sprawa_nieruchomosci_wpisy (id, rodzaj, prawo, miejscowosc, gmina, ulica, numer_budynku,
  numer_lokalu, kod_pocztowy, numer_kw, kw_brak, powierzchnia_uzytkowa, liczba_pokoi, pietro,
  nadrzedna_id)
VALUES (:id, :rodzaj, :prawo, :miejscowosc, :gmina, :ulica, :numer_budynku, :numer_lokalu,
  :kod_pocztowy, :numer_kw, :kw_brak, :powierzchnia_uzytkowa, :liczba_pokoi, :pietro, :nadrzedna_id);

-- name: zmien_dane_nieruchomosci :exec
-- Poprawka oznaczenia, np. po odczytaniu odpisu księgi. Rodzaj się nie zmienia; nieruchomość
-- innego rodzaju to nowy wiersz.
UPDATE sprawa_nieruchomosci_wpisy
SET prawo = :prawo,
  miejscowosc = :miejscowosc,
  gmina = :gmina,
  ulica = :ulica,
  numer_budynku = :numer_budynku,
  numer_lokalu = :numer_lokalu,
  kod_pocztowy = :kod_pocztowy,
  numer_kw = :numer_kw,
  kw_brak = :kw_brak,
  powierzchnia_uzytkowa = :powierzchnia_uzytkowa,
  liczba_pokoi = :liczba_pokoi,
  pietro = :pietro,
  nadrzedna_id = :nadrzedna_id
WHERE id = :id;

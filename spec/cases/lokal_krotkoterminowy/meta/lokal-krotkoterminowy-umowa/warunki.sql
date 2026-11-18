-- name: warunki_umowy_lokalu :many
-- Wersje warunków umowy o zarządzanie w kolejności chwili, od której obowiązują.
SELECT *
FROM lokal_krotkoterminowy_umowa_warunki
ORDER BY od, rowid;

-- name: biezace_warunki_umowy_lokalu :one
-- Ustalona wersja warunków z najpóźniejszą chwilą od; z nią porównują się pobyty, zlecenia i ceny.
SELECT *
FROM lokal_krotkoterminowy_umowa_warunki_biezace;

-- name: umowa_lokalu :one
-- Zawarta albo zakończona umowa o zarządzanie lokalem z datą końca i chwilą praca_od.
-- column praca_od: string
SELECT *
FROM lokal_krotkoterminowy_umowa_lokalu;

-- name: wpisz_warunki_umowy_lokalu :exec
-- Projekt wersji warunków: pierwsza od zawarcia umowy, następna od aneksu. Identyfikator (UUID)
-- nadaje aplikacja, bo wskazują go kanały i widełki.
INSERT INTO lokal_krotkoterminowy_umowa_warunki (id, umowa_id, max_gosci, prowizja_rodzaj, prowizja,
  prowizja_podstawa, oplata_sprzatanie, oplata_sprzatanie_pokrywa_koszt, kwota_napraw,
  koszty_wlasciciela, zgloszenie_nocy_dni, wyplata_dzien, rezerwa, po_koncu_rezerwacje,
  aneks_zdarzenie_id, od, stan)
VALUES (:id, :umowa_id, :max_gosci, :prowizja_rodzaj, :prowizja, :prowizja_podstawa,
  :oplata_sprzatanie, :oplata_sprzatanie_pokrywa_koszt, :kwota_napraw, :koszty_wlasciciela,
  :zgloszenie_nocy_dni, :wyplata_dzien, :rezerwa, :po_koncu_rezerwacje, :aneks_zdarzenie_id, :od,
  'projekt');

-- name: ustal_warunki_umowy_lokalu :exec
-- Projekt z kanałami i widełkami zaczyna obowiązywać; potem się go nie zmienia.
UPDATE lokal_krotkoterminowy_umowa_warunki
SET stan = 'obowiazuje'
WHERE id = :id;

-- name: warunki_najmu :many
-- Wersje warunków najmu od najstarszej.
SELECT *
FROM oferta_najem_warunki
ORDER BY rowid;

-- name: warunki_najmu_biezace :one
-- Warunki najmu, które obowiązują: wersja, której nie zastąpiła nowsza.
SELECT *
FROM oferta_najem_warunki AS warunki
WHERE NOT EXISTS (
  SELECT 1
  FROM oferta_najem_warunki AS nowsze
  WHERE nowsze.zastepuje_id = warunki.id
);

-- name: wpisz_warunki_najmu :exec
-- Warunki najmu zatwierdzone przez klienta; nowa wersja zastępuje obowiązującą. Identyfikator
-- (UUID) nadaje aplikacja, bo wskazuje go zgoda klienta.
INSERT INTO oferta_najem_warunki (id, oplaty_obok_czynszu, rozliczenie_mediow, kaucja,
  dopuszcza_zwykly, dopuszcza_okazjonalny, dopuszcza_instytucjonalny, wolny_od, okres_min,
  okres_max, wyposazenie, liczba_osob, zwierzeta, palenie, prowadzenie_najmu, zgoda_id, zastepuje_id)
VALUES (:id, :oplaty_obok_czynszu, :rozliczenie_mediow, :kaucja, :dopuszcza_zwykly,
  :dopuszcza_okazjonalny, :dopuszcza_instytucjonalny, :wolny_od, :okres_min, :okres_max,
  :wyposazenie, :liczba_osob, :zwierzeta, :palenie, :prowadzenie_najmu, :zgoda_id, :zastepuje_id);

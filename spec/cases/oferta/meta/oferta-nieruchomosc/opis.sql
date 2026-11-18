-- name: opisy_nieruchomosci :many
-- Opisy nieruchomości oferty: budynek, działka, koszty, świadectwo i osoby w lokalu.
SELECT *
FROM oferta_nieruchomosc_opis
ORDER BY nieruchomosc_id;

-- name: wpisz_opis_nieruchomosci :exec
-- Opis nieruchomości z rdzenia; źródło każdej danej wpisuje się w sprawa_dokumenty_zrodla.
INSERT INTO oferta_nieruchomosc_opis (nieruchomosc_id, liczba_pieter, winda, rok_budowy, material,
  ogrzewanie, stan_lokalu, remonty, przeznaczenie_dzialki, dostep_do_drogi, media, uzytki_rolne_m2,
  oplata_miesieczna, media_srednio, podatek, swiadectwo, swiadectwo_dokument_id,
  wskazniki_energetyczne, osoby_w_lokalu, wynajety_do, przeznaczenie_lokalu_uzytkowego)
VALUES (:nieruchomosc_id, :liczba_pieter, :winda, :rok_budowy, :material, :ogrzewanie, :stan_lokalu,
  :remonty, :przeznaczenie_dzialki, :dostep_do_drogi, :media, :uzytki_rolne_m2, :oplata_miesieczna,
  :media_srednio, :podatek, :swiadectwo, :swiadectwo_dokument_id, :wskazniki_energetyczne,
  :osoby_w_lokalu, :wynajety_do, :przeznaczenie_lokalu_uzytkowego);

-- name: zmien_opis_nieruchomosci :exec
-- Nowy stan opisu, np. po sprawdzeniu danej od klienta z dokumentem albo po nowym świadectwie.
UPDATE oferta_nieruchomosc_opis
SET liczba_pieter = :liczba_pieter,
  winda = :winda,
  rok_budowy = :rok_budowy,
  material = :material,
  ogrzewanie = :ogrzewanie,
  stan_lokalu = :stan_lokalu,
  remonty = :remonty,
  przeznaczenie_dzialki = :przeznaczenie_dzialki,
  dostep_do_drogi = :dostep_do_drogi,
  media = :media,
  uzytki_rolne_m2 = :uzytki_rolne_m2,
  oplata_miesieczna = :oplata_miesieczna,
  media_srednio = :media_srednio,
  podatek = :podatek,
  swiadectwo = :swiadectwo,
  swiadectwo_dokument_id = :swiadectwo_dokument_id,
  wskazniki_energetyczne = :wskazniki_energetyczne,
  osoby_w_lokalu = :osoby_w_lokalu,
  wynajety_do = :wynajety_do,
  przeznaczenie_lokalu_uzytkowego = :przeznaczenie_lokalu_uzytkowego
WHERE nieruchomosc_id = :nieruchomosc_id;

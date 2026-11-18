-- name: dane_wykonawcy :many
-- Wszystkie wersje danych wykonawcy, najnowsza pierwsza; aktualna ma aktualne = 1.
SELECT *
FROM wykonawcy_spis_dane
WHERE wykonawca_id = :wykonawca_id
ORDER BY id DESC;

-- name: wpisz_dane_wykonawcy :exec
-- Nowa wersja danych po decyzji o zmianie danych, np. przedłużona umowa albo nowa polisa.
INSERT INTO wykonawcy_spis_dane (wykonawca_id, aktualne, firma, nip, krs, ceidg, adres, zakres_uslug,
  srodki_wlasne, umowa_od, umowa_do, umowa_podpisal, umowa_sciezka, umowa_skrot, polisa_do,
  polisa_sciezka, polisa_skrot, decyzja_id, wpisano)
VALUES (:wykonawca_id, 1, :firma, :nip, :krs, :ceidg, :adres, :zakres_uslug, :srodki_wlasne,
  :umowa_od, :umowa_do, :umowa_podpisal, :umowa_sciezka, :umowa_skrot, :polisa_do,
  :polisa_sciezka, :polisa_skrot, :decyzja_id, :wpisano);

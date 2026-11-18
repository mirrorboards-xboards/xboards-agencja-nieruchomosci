-- name: dziennik_dnia :many
-- Wpisy dziennika z jednego dnia (RRRR-MM-DD), od najwcześniejszego.
-- param dzien: string
SELECT *
FROM recepcja_wplywy_dziennik
WHERE substr(wplynelo, 1, 10) = :dzien
ORDER BY wplynelo, id;

-- name: wplywy_sprawy :many
-- Wszystkie wiadomości jednej sprawy, od najwcześniejszej.
SELECT *
FROM recepcja_wplywy_dziennik
WHERE sprawa_id = :sprawa_id
ORDER BY wplynelo, id;

-- name: wplywy_od_nadawcy :many
-- Wiadomości od jednego nadawcy, od najwcześniejszej; z nich widać, czy kanał do niego pasuje.
SELECT *
FROM recepcja_wplywy_dziennik
WHERE nadawca = :nadawca
ORDER BY wplynelo, id;

-- name: wplywy_wedlug_kanalow :many
-- Ile wiadomości wpłynęło jednego dnia (RRRR-MM-DD) każdym kanałem, do meldunku.
-- param dzien: string
-- column liczba: integer
SELECT kanal,
  count(*) AS liczba
FROM recepcja_wplywy_dziennik
WHERE substr(wplynelo, 1, 10) = :dzien
GROUP BY kanal
ORDER BY kanal;

-- name: odnotuj_wplyw :exec
-- Wiadomość rozdzielona: sprawa i znak albo wpis katalogu, jeśli pasują, oraz komu ją oddano.
INSERT INTO recepcja_wplywy_dziennik (wplynelo, kanal, nadawca, rodzaj_pisma, wiadomosc_odnosnik,
  sprawa_id, znak_id, katalog_odnosnik, oddano, oddano_komu, oddano_o, kanal_nie_pasuje)
VALUES (:wplynelo, :kanal, :nadawca, :rodzaj_pisma, :wiadomosc_odnosnik, :sprawa_id, :znak_id,
  :katalog_odnosnik, :oddano, :oddano_komu, :oddano_o, :kanal_nie_pasuje);

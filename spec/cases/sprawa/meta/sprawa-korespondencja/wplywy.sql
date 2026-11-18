-- name: wplywy :many
-- Wpływy teczki, najnowsze pierwsze.
SELECT *
FROM sprawa_korespondencja_wplywy
ORDER BY wplynelo DESC, id;

-- name: wpisz_wplyw :exec
-- Wiadomość z zewnątrz z nadawcą, kanałem, czasem i plikiem; gdy nadawca albo kanał nie pasuje do
-- kontaktów w teczce, agent zaznacza to z opisem.
INSERT INTO sprawa_korespondencja_wplywy (id, nadawca, nadawca_uczestnik_id, kanal, wplynelo,
  sciezka, skrot, niezgodny_kontakt, niezgodnosc_opis, odnosnik_zewnetrzny)
VALUES (:id, :nadawca, :nadawca_uczestnik_id, :kanal, :wplynelo, :sciezka, :skrot,
  :niezgodny_kontakt, :niezgodnosc_opis, :odnosnik_zewnetrzny);

-- name: dostepy_do_lokalu :many
-- Kody i klucze w kolejności początku ważności.
SELECT *
FROM lokal_krotkoterminowy_zlecenia_dostepy
ORDER BY wazny_od, id;

-- name: ustaw_dostep :exec
-- Kod do zamka albo klucze dla pobytu albo zlecenia, z czasem ważności i adresatem.
INSERT INTO lokal_krotkoterminowy_zlecenia_dostepy (pobyt_id, zlecenie_id, rodzaj, kod, klucze_liczba,
  zamek_odnosnik, wazny_od, wazny_do, kto, ustawiono)
VALUES (:pobyt_id, :zlecenie_id, :rodzaj, :kod, :klucze_liczba, :zamek_odnosnik, :wazny_od,
  :wazny_do, :kto, :ustawiono);

-- name: odnotuj_wydanie_dostepu :exec
-- Kod wysłany gościowi albo wykonawcy, albo klucze wydane; wpisuje się raz.
-- param wydano: string
UPDATE lokal_krotkoterminowy_zlecenia_dostepy
SET wydano = :wydano
WHERE id = :id;

-- name: odnotuj_odebranie_dostepu :exec
-- Kod wygaszony albo klucze odebrane; wpisuje się raz.
-- param odebrano: string
UPDATE lokal_krotkoterminowy_zlecenia_dostepy
SET odebrano = :odebrano
WHERE id = :id;

-- name: analizy_kredytowe :many
-- Analizy kredytowe, najnowsze pierwsze.
SELECT *
FROM poszukiwanie_kredyt_analizy
ORDER BY utworzono DESC, id;

-- name: analiza_aktualna :one
-- Analiza, która obowiązuje; poprzednie straciły aktualność.
SELECT *
FROM poszukiwanie_kredyt_analizy
WHERE aktualna = 1;

-- name: dane_analiz :many
-- Dane wejściowe wprowadzone do programu w każdej analizie.
SELECT *
FROM poszukiwanie_kredyt_analizy_dane
ORDER BY analiza_id, id;

-- name: wpisz_analize_kredytowa :exec
-- Wynik programu z wersją parametrów, wyjaśnieniem i zastrzeżeniem (R2); poprzednie analizy
-- tracą aktualność. Analiza poza standardem wskazuje zatwierdzoną decyzję człowieka.
INSERT INTO poszukiwanie_kredyt_analizy (id, wersja_parametrow, okres_lat, poza_standardem,
  poza_standardem_decyzja_id, kwota_od, kwota_do, rata_od, rata_do, udzial_wkladu, wynik_sciezka,
  wynik_skrot, wyjasnienie, zastrzezenie, aktualna, utworzono)
VALUES (:id, :wersja_parametrow, :okres_lat, :poza_standardem, :poza_standardem_decyzja_id,
  :kwota_od, :kwota_do, :rata_od, :rata_do, :udzial_wkladu, :wynik_sciezka, :wynik_skrot,
  :wyjasnienie, :zastrzezenie, 1, :utworzono);

-- name: dopisz_dana_do_analizy :exec
-- Dana z dokumentu, którą Analityk wprowadził do programu w tej analizie.
INSERT INTO poszukiwanie_kredyt_analizy_dane (analiza_id, dana_id)
VALUES (:analiza_id, :dana_id);

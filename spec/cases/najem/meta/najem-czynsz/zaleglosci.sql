-- name: zaleglosci :many
-- Zaległości za co najmniej trzy pełne okresy z uprzedzeniem, terminem i wynikiem.
SELECT *
FROM najem_czynsz_zaleglosci
ORDER BY trzy_okresy_od, id;

-- name: wpisz_zaleglosc :exec
-- Dzień, w którym zaległość doszła do trzech pełnych okresów płatności; liczy program.
INSERT INTO najem_czynsz_zaleglosci (id, najem_id, trzy_okresy_od, wynik)
VALUES (:id, :najem_id, :trzy_okresy_od, 'czeka');

-- name: wskaz_uprzedzenie_o_zaleglosci :exec
-- Pismo o zamiarze wypowiedzenia, jeszcze przed wysłaniem; wychodzi za zgodą człowieka.
-- param uprzedzenie_wysylka_id: string
UPDATE najem_czynsz_zaleglosci
SET uprzedzenie_wysylka_id = :uprzedzenie_wysylka_id
WHERE id = :id;

-- name: ustal_dodatkowy_termin_zaleglosci :exec
-- Koniec dodatkowego terminu na zapłatę, co najmniej miesiąc od doręczenia uprzedzenia.
-- param dodatkowy_termin_do: string
UPDATE najem_czynsz_zaleglosci
SET dodatkowy_termin_do = :dodatkowy_termin_do
WHERE id = :id;

-- name: odnotuj_wynik_zaleglosci :exec
-- Najemca spłacił zaległość albo wynajmujący wypowiada najem po dodatkowym terminie.
-- param wynik: "splacona" | "wypowiedzenie"
UPDATE najem_czynsz_zaleglosci
SET wynik = :wynik
WHERE id = :id;

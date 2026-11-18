-- name: sprawy_w_spisie :many
-- Wszystkie sprawy w spisie; najpierw otwarte, potem zamknięte.
SELECT *
FROM kierownik_dzialu_spis_sprawy
ORDER BY zamknieto IS NOT NULL, oznaczenie, id;

-- name: do_przekazania_recepcji :many
-- Sprawy, których stanu Recepcja jeszcze nie dostała: nowe, zmienione albo zamknięte. Znaki do
-- przekazania podaje znaki_do_przekazania_recepcji.
SELECT *
FROM kierownik_dzialu_spis_sprawy
WHERE przekazano IS NULL
  OR (zamknieto IS NOT NULL AND zamkniecie_przekazano IS NULL)
ORDER BY id;

-- name: wpisz_sprawe_do_spisu :exec
-- Nowa sprawa działu w spisie, gdy kierownik powołał jej agenta; przy transakcji z informacją, czy
-- kancelaria pracuje na XNotariuszu.
INSERT INTO kierownik_dzialu_spis_sprawy (teczka_odnosnik, rodzaj, oznaczenie, agent, otwarto, kancelaria_xnotariusz)
VALUES (:teczka_odnosnik, :rodzaj, :oznaczenie, :agent, :otwarto, :kancelaria_xnotariusz);

-- name: zmien_sprawe_w_spisie :exec
-- Sprawa się zmieniła, np. ma nowe oznaczenie albo agenta po wymianie; nowy stan czeka na
-- przekazanie Recepcji.
UPDATE kierownik_dzialu_spis_sprawy
SET oznaczenie = :oznaczenie,
  agent = :agent,
  kancelaria_xnotariusz = :kancelaria_xnotariusz,
  przekazano = NULL
WHERE id = :id;

-- name: odnotuj_przekazanie_sprawy :exec
-- Recepcja dostała bieżący stan sprawy.
-- param przekazano: string
UPDATE kierownik_dzialu_spis_sprawy
SET przekazano = :przekazano
WHERE id = :id;

-- name: zamknij_sprawe_w_spisie :exec
-- Sprawa się zamknęła; zostaje w spisie, a zamknięcie czeka na przekazanie Recepcji.
-- param zamknieto: string
UPDATE kierownik_dzialu_spis_sprawy
SET zamknieto = :zamknieto
WHERE id = :id;

-- name: odnotuj_przekazanie_zamkniecia :exec
-- Recepcja dostała zamknięcie sprawy.
-- param zamkniecie_przekazano: string
UPDATE kierownik_dzialu_spis_sprawy
SET zamkniecie_przekazano = :zamkniecie_przekazano
WHERE id = :id;

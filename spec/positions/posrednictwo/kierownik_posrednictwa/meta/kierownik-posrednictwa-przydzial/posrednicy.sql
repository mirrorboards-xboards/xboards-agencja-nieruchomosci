-- name: posrednicy :many
-- Aktualne reguły pośredników.
SELECT *
FROM kierownik_posrednictwa_przydzial_posrednicy
WHERE zastapione_od IS NULL
ORDER BY posrednik, id;

-- name: historia_regul_posrednika :many
-- Wszystkie reguły jednego pośrednika, także zastąpione, od najstarszej.
SELECT *
FROM kierownik_posrednictwa_przydzial_posrednicy
WHERE posrednik = :posrednik
ORDER BY od, id;

-- name: obciazenie_posrednikow :many
-- Ile spraw prowadzi każdy pośrednik z aktualną regułą wobec swojego limitu: tyle, ilu niewymienionych
-- opiekunów ma jako właściciel. Według tego kierownik proponuje pośrednika i melduje obłożenie.
-- column sprawy: integer
SELECT regula.id AS regula_id,
  regula.posrednik,
  regula.limit_spraw,
  regula.przyjmuje_nowe,
  (
    SELECT count(*)
    FROM kierownik_dzialu_agenci_wpisy AS agent
    WHERE agent.wlasciciel = regula.posrednik
      AND agent.stanowisko IN ('opiekun_oferty', 'opiekun_poszukujacego')
      AND agent.stan <> 'wymieniony'
  ) AS sprawy
FROM kierownik_posrednictwa_przydzial_posrednicy AS regula
WHERE regula.zastapione_od IS NULL
ORDER BY regula.posrednik, regula.id;

-- name: zapisz_regule_posrednika :exec
-- Nowa reguła z podpisanej wiadomości pośrednika; poprzednią trzeba najpierw zastąpić.
INSERT INTO kierownik_posrednictwa_przydzial_posrednicy (posrednik, obszar, rodzaje_nieruchomosci,
  limit_spraw, przyjmuje_nowe, od, wiadomosc_odnosnik, podpis)
VALUES (:posrednik, :obszar, :rodzaje_nieruchomosci, :limit_spraw, :przyjmuje_nowe, :od,
  :wiadomosc_odnosnik, :podpis);

-- name: zastap_regule_posrednika :exec
-- Reguła przestaje obowiązywać, bo pośrednik podpisał nową.
-- param zastapione_od: string
UPDATE kierownik_posrednictwa_przydzial_posrednicy
SET zastapione_od = :zastapione_od
WHERE id = :id;

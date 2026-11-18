-- name: zdarzenia_zrodla :many
-- Zdarzenia jednego źródła, od najstarszego; na nich stoi jego ocena.
SELECT *
FROM stanowisko_zrodla_zdarzenia
WHERE zrodlo_id = :zrodlo_id
ORDER BY zaszlo, id;

-- name: zdarzenia_do_zgloszenia :many
-- Zdarzenia, których agent nie podał jeszcze w raporcie; z nich składa część raportu o źródłach.
SELECT *
FROM stanowisko_zrodla_zdarzenia
WHERE zgloszono IS NULL
ORDER BY zaszlo, id;

-- name: dopisz_zdarzenie_zrodla :exec
-- Zdarzenie ze źródłem: fakt, zakłócenie albo rozstrzygnięcie zarzutu.
INSERT INTO stanowisko_zrodla_zdarzenia (zrodlo_id, zaszlo, rodzaj, opis, dotyczy_tabela, dotyczy_odnosnik)
VALUES (:zrodlo_id, :zaszlo, :rodzaj, :opis, :dotyczy_tabela, :dotyczy_odnosnik);

-- name: odnotuj_zgloszenie_zdarzenia :exec
-- Zdarzenie weszło do raportu.
-- param zgloszono: string
UPDATE stanowisko_zrodla_zdarzenia
SET zgloszono = :zgloszono
WHERE id = :id;

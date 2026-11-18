-- name: zmiany_czynszu :many
-- Zmiany czynszu w kolejności wpisania; bieżący czynsz to nowa kwota ostatniej.
SELECT *
FROM najem_czynsz_zmiany
ORDER BY najem_id, id;

-- name: wpisz_zmiane_czynszu :exec
-- Zmiana czynszu: wypowiedzenie wysokości z doręczeniem albo obowiązujący aneks, z podstawą.
INSERT INTO najem_czynsz_zmiany (najem_id, stara, nowa, pismo_wysylka_id, doreczono, obowiazuje_od,
  podstawa, aneks_id)
VALUES (:najem_id, :stara, :nowa, :pismo_wysylka_id, :doreczono, :obowiazuje_od, :podstawa,
  :aneks_id);

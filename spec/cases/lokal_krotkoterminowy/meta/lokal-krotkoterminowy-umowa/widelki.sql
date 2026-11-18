-- name: widelki_warunkow :many
-- Widełki cen każdej wersji warunków w kolejności okresów.
SELECT *
FROM lokal_krotkoterminowy_umowa_widelki
ORDER BY warunki_id, od_dnia;

-- name: dopisz_widelki_warunkow :exec
-- Najniższa i najwyższa cena nocy oraz najkrótszy pobyt w okresie, w projekcie warunków.
INSERT INTO lokal_krotkoterminowy_umowa_widelki (warunki_id, sezon, od_dnia, do_dnia, cena_min,
  cena_max, pobyt_min)
VALUES (:warunki_id, :sezon, :od_dnia, :do_dnia, :cena_min, :cena_max, :pobyt_min);

-- name: strony_aktu :many
-- Strony aktu w kolejności wpisania, z tym, które reprezentuje biuro.
SELECT *
FROM wymiana_metryka_strony
ORDER BY rowid;

-- name: wpisz_strone_aktu :exec
-- Biuro wpisuje stronę aktu przy otwarciu pokoju albo gdy strona dojdzie później.
INSERT INTO wymiana_metryka_strony (id, nazwa, rola, reprezentowana_przez_biuro)
VALUES (:id, :nazwa, :rola, :reprezentowana_przez_biuro);

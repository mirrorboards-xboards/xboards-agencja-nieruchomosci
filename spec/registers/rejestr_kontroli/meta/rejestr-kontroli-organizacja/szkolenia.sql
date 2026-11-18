-- name: szkolenia :many
-- Szkolenia, najnowsze pierwsze.
SELECT *
FROM rejestr_kontroli_organizacja_szkolenia
ORDER BY odbyto DESC, id;

-- name: wpisz_szkolenie :exec
-- Szkolenie po potwierdzeniu udziału przez człowieka.
INSERT INTO rejestr_kontroli_organizacja_szkolenia (uczestnik, odbyto, temat, potwierdzil, wpisal,
  wpisano)
VALUES (:uczestnik, :odbyto, :temat, :potwierdzil, :wpisal, :wpisano);

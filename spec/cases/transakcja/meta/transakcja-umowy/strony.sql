-- name: strony_umow_transakcji :many
-- Strony wszystkich umów transakcji z umocowaniem pełnomocnika, jeśli jest.
SELECT *
FROM transakcja_umowy_strony
ORDER BY umowa_id, id;

-- name: dopisz_strone_umowy_transakcji :exec
-- Sprzedający, kupujący albo małżonek, który wyraża zgodę; przy pełnomocniku jego umocowanie.
INSERT INTO transakcja_umowy_strony (umowa_id, uczestnik_id, strona, umocowanie_id)
VALUES (:umowa_id, :uczestnik_id, :strona, :umocowanie_id);

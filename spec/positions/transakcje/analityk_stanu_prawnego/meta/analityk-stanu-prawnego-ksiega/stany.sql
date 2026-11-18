-- name: stany_ksiegi :many
-- Stany jednej księgi, od najnowszego.
SELECT *
FROM analityk_stanu_prawnego_ksiega_stany
WHERE numer_kw = :numer_kw
ORDER BY chwila DESC, id;

-- name: zapisz_stan_ksiegi :exec
-- Nowy odpis albo przeglądanie księgi; wpisy z tej chwili dopisuje zapisz_wpis_ksiegi.
INSERT INTO analityk_stanu_prawnego_ksiega_stany (numer_kw, chwila, sposob, dokument_odnosnik)
VALUES (:numer_kw, :chwila, :sposob, :dokument_odnosnik);

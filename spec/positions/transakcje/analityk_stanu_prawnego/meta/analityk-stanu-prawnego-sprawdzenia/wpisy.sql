-- name: sprawdzenia_stanu_prawnego :many
-- Aktualne sprawdzenia (niezastąpione); najpierw ryzyka i to, czego nie można ustalić.
SELECT *
FROM analityk_stanu_prawnego_sprawdzenia_wpisy
WHERE zastapione_przez_id IS NULL
ORDER BY wynik = 'bez_uwag', sprawdzono DESC, id;

-- name: sprawdzenia_dokumentu :many
-- Aktualne sprawdzenia, które stoją na jednym dokumencie; po nowej wersji dokumentu Analityk
-- sprawdza je ponownie.
SELECT *
FROM analityk_stanu_prawnego_sprawdzenia_wpisy
WHERE dokument_odnosnik = :dokument_odnosnik
  AND zastapione_przez_id IS NULL
ORDER BY sprawdzono, id;

-- name: sprawdzenia_bez_analizy :many
-- Aktualne sprawdzenia, które jeszcze nie weszły do analizy w teczce.
SELECT *
FROM analityk_stanu_prawnego_sprawdzenia_wpisy
WHERE analiza_odnosnik IS NULL
  AND zastapione_przez_id IS NULL
ORDER BY sprawdzono, id;

-- name: zapisz_sprawdzenie_stanu_prawnego :exec
-- Sprawdzenie z dokumentem, przepisem i wynikiem; identyfikator (UUID) nadaje aplikacja, żeby
-- poprzednie sprawdzenie mogło go wskazać jako zastępujące.
INSERT INTO analityk_stanu_prawnego_sprawdzenia_wpisy (id, przedmiot, dokument_odnosnik,
  dokument_data, przepis, wynik, opis, ksiega_stan_id, sprawdzono)
VALUES (:id, :przedmiot, :dokument_odnosnik, :dokument_data, :przepis, :wynik, :opis,
  :ksiega_stan_id, :sprawdzono);

-- name: odnotuj_analize_sprawdzenia :exec
-- Sprawdzenie weszło do analizy w teczce transakcji.
-- param analiza_odnosnik: string
UPDATE analityk_stanu_prawnego_sprawdzenia_wpisy
SET analiza_odnosnik = :analiza_odnosnik
WHERE id = :id;

-- name: zastap_sprawdzenie_stanu_prawnego :exec
-- Poprzednie sprawdzenie staje się nieaktualne, bo zastępuje je ponowne, zapisane wcześniej.
-- param zastapione_przez_id: string
UPDATE analityk_stanu_prawnego_sprawdzenia_wpisy
SET zastapione_przez_id = :zastapione_przez_id
WHERE id = :id;

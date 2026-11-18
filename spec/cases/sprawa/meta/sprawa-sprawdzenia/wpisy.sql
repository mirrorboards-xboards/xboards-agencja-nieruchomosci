-- name: sprawdzenia :many
-- Wyniki sprawdzeń Kontrolera, najnowsze pierwsze.
SELECT *
FROM sprawa_sprawdzenia_wpisy
ORDER BY sprawdzono DESC, id;

-- name: sprawdzenia_aktualne :many
-- Wyniki, które obejmują bieżącą wersję sprawdzanych wierszy.
SELECT *
FROM sprawa_sprawdzenia_wpisy
WHERE aktualne = 1
ORDER BY sprawdzono DESC, id;

-- name: wpisz_sprawdzenie :exec
-- Wynik sprawdzenia Kontrolera dla wiersza w bieżącej wersji; poprzedni wynik tego wiersza
-- traci aktualność.
INSERT INTO sprawa_sprawdzenia_wpisy (id, rodzaj, uczestnik_id, nieruchomosc_id, dokument_id,
  umocowanie_id, rachunek_id, wersja_wiersza, zrodlo, zrodlo_data, wynik, opis, sprawdzil,
  sprawdzono, aktualne)
VALUES (:id, :rodzaj, :uczestnik_id, :nieruchomosc_id, :dokument_id, :umocowanie_id, :rachunek_id,
  :wersja_wiersza, :zrodlo, :zrodlo_data, :wynik, :opis, :sprawdzil, :sprawdzono, 1);

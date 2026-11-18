-- name: wpisy :many
-- Wszystkie wpisy pokoju w kolejności powstania; poprawione zostają obok poprawek.
SELECT *
FROM wymiana_wpisy_wpisy
ORDER BY rowid;

-- name: wpis :one
-- Jeden wpis, np. ten, na który odpowiada wiadomość albo który wskazuje teczka.
SELECT *
FROM wymiana_wpisy_wpisy
WHERE id = :id;

-- name: wpisy_od_chwili :many
-- Wpisy, które powstały od danej chwili, np. od ostatniego przeniesienia do teczki.
-- param od: string
SELECT *
FROM wymiana_wpisy_wpisy
WHERE chwila >= :od
ORDER BY rowid;

-- name: wpisz_wiadomosc :exec
-- Pytanie albo odpowiedź, która nie mieści się w polach innych wpisów; wskazuje wpis, którego
-- dotyczy. Może mieć plik.
INSERT INTO wymiana_wpisy_wpisy (id, strona, autor, rodzaj, chwila, odpowiada_na_id, poprawia_id,
  tresc, sciezka, skrot)
VALUES (:id, :strona, :autor, 'wiadomosc', :chwila, :odpowiada_na_id, :poprawia_id, :tresc,
  :sciezka, :skrot);

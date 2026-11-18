-- name: cenniki :many
-- Cenniki rejestru w kolejności wpisania.
SELECT *
FROM wykonawcy_cenniki_wpisy
ORDER BY rowid;

-- name: wpisz_cennik :exec
-- Nowy cennik; jego stawki to pierwsza wersja. Identyfikator (UUID) nadaje aplikacja.
INSERT INTO wykonawcy_cenniki_wpisy (id, nazwa, wpisano)
VALUES (:id, :nazwa, :wpisano);

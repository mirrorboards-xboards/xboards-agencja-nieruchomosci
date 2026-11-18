-- name: analizy_klienta :many
-- Wyniki bieżącej analizy transakcji klienta, najnowsze pierwsze.
SELECT *
FROM rejestr_kontroli_sprawdzenia_monitorowanie
WHERE klient_id = :klient_id
ORDER BY id DESC;

-- name: wpisz_analize :exec
-- Wynik bieżącej analizy transakcji klienta w jednej teczce.
INSERT INTO rejestr_kontroli_sprawdzenia_monitorowanie (klient_id, teczka_odnosnik, analiza, wynik,
  opis, zrodlo, raport_odnosnik, sprawdzil, sprawdzono, wpisal, wpisano)
VALUES (:klient_id, :teczka_odnosnik, :analiza, :wynik, :opis, :zrodlo, :raport_odnosnik,
  :sprawdzil, :sprawdzono, :wpisal, :wpisano);

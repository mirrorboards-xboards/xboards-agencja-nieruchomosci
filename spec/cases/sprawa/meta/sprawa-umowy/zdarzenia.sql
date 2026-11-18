-- name: zdarzenia_umow :many
-- Zdarzenia umów teczki w kolejności czasu.
SELECT *
FROM sprawa_umowy_zdarzenia
ORDER BY chwila, id;

-- name: wpisz_zdarzenie_umowy :exec
-- Aneks, polisa, wezwanie, żądanie, odstąpienie, wypowiedzenie albo zdarzenie wynagrodzenia,
-- z dokumentem, wysyłką, wpływem albo zgodą, które je dowodzą; tylko przy zawartej albo
-- zakończonej umowie.
INSERT INTO sprawa_umowy_zdarzenia (id, umowa_id, rodzaj, chwila, opis, dokument_id, wysylka_id,
  wplyw_id, zgoda_id)
VALUES (:id, :umowa_id, :rodzaj, :chwila, :opis, :dokument_id, :wysylka_id, :wplyw_id, :zgoda_id);

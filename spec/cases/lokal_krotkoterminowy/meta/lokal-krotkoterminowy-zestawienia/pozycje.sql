-- name: pozycje_zestawien :many
-- Pozycje wszystkich zestawień w kolejności wpisania.
SELECT *
FROM lokal_krotkoterminowy_zestawienia_pozycje
ORDER BY zestawienie_id, id;

-- name: dopisz_pozycje_zestawienia :exec
-- Pozycja ze znakiem i źródłem w teczce, w projekcie zestawienia.
INSERT INTO lokal_krotkoterminowy_zestawienia_pozycje (zestawienie_id, rodzaj, kwota, opis, pobyt_id,
  zlecenie_id, pieniadze_id, warunki_id, dokument_id)
VALUES (:zestawienie_id, :rodzaj, :kwota, :opis, :pobyt_id, :zlecenie_id, :pieniadze_id, :warunki_id,
  :dokument_id);

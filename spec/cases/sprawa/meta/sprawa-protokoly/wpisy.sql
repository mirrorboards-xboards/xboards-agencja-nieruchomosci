-- name: protokoly :many
-- Protokoły teczki w kolejności chwili przekazania.
SELECT *
FROM sprawa_protokoly_wpisy
ORDER BY chwila, id;

-- name: przygotuj_protokol :exec
-- Projekt protokołu przekazania nieruchomości; identyfikator (UUID) nadaje aplikacja, bo wskazują
-- go pozycje i wiersze typu sprawy.
INSERT INTO sprawa_protokoly_wpisy (id, rodzaj, nieruchomosc_id, chwila, stan)
VALUES (:id, :rodzaj, :nieruchomosc_id, :chwila, 'projekt');

-- name: podpisz_protokol :exec
-- Protokół podpisany przez strony; dokument to otrzymany plik protokołu z podpisami.
-- param dokument_id: string
UPDATE sprawa_protokoly_wpisy
SET stan = 'podpisany',
  dokument_id = :dokument_id
WHERE id = :id;

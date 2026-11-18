-- name: analizy_transakcji :many
-- Analizy stanu prawnego, najnowsze pierwsze.
SELECT *
FROM transakcja_analizy_wpisy
ORDER BY sporzadzono DESC, id;

-- name: analizy_aktualne_transakcji :many
-- Bieżąca ocena: aktualna analiza każdego przedmiotu.
SELECT *
FROM transakcja_analizy_wpisy
WHERE aktualna = 1
ORDER BY przedmiot, umowa_id;

-- name: wpisz_analize_transakcji :exec
-- Nowa analiza z wnioskami i plikiem; poprzednia analiza tego przedmiotu traci aktualność.
INSERT INTO transakcja_analizy_wpisy (id, przedmiot, umowa_id, dokument_id,
  poprzedni_odpis_dokument_id, sporzadzono, wnioski, sciezka, skrot, aktualna)
VALUES (:id, :przedmiot, :umowa_id, :dokument_id, :poprzedni_odpis_dokument_id, :sporzadzono,
  :wnioski, :sciezka, :skrot, 1);

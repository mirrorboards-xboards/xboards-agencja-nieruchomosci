-- name: notatki :many
-- Notatki przekazania w teczce, najnowsze pierwsze.
SELECT *
FROM sprawa_notatki_wpisy
ORDER BY utworzono DESC, id;

-- name: notatki_otwarte_do_stanowiska :many
-- Otwarte notatki dla jednego stanowiska, najstarsze pierwsze.
SELECT *
FROM sprawa_notatki_wpisy
WHERE do_stanowiska = :do_stanowiska
  AND stan = 'otwarta'
ORDER BY utworzono, id;

-- name: dopisz_notatke :exec
-- Notatka przekazania: co się stało, czego potrzeba i do kiedy; identyfikator (UUID) nadaje
-- aplikacja, żeby od razu wskazać wiersze, których dotyczy.
INSERT INTO sprawa_notatki_wpisy (id, od, od_stanowiska, do_stanowiska, do_klienta_uczestnik_id,
  tresc, utworzono, do_kiedy, w_odpowiedzi_na, stan)
VALUES (:id, :od, :od_stanowiska, :do_stanowiska, :do_klienta_uczestnik_id, :tresc, :utworzono,
  :do_kiedy, :w_odpowiedzi_na, 'otwarta');

-- name: oznacz_notatke_jako_wykonana :exec
-- Adresat wykonał prośbę z notatki.
UPDATE sprawa_notatki_wpisy
SET stan = 'wykonana'
WHERE id = :id;

-- name: odrzuc_notatke :exec
-- Adresat nie wykona prośby, bo wykracza poza jego normy; powód wraca do autora.
-- param powod: string
UPDATE sprawa_notatki_wpisy
SET stan = 'odrzucona',
  powod = :powod
WHERE id = :id;

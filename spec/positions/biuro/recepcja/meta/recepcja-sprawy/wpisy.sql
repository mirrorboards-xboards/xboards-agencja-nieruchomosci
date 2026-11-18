-- name: spis_spraw :many
-- Spis spraw Recepcji; otwarte pierwsze, potem według oznaczenia.
SELECT *
FROM recepcja_sprawy_wpisy
ORDER BY stan = 'zamknieta', oznaczenie, id;

-- name: wpisz_sprawe :exec
-- Nowa sprawa przekazana przez kierownika działu.
INSERT INTO recepcja_sprawy_wpisy (teczka_odnosnik, rodzaj, oznaczenie, agent, stan, otwarto,
  kancelaria_xnotariusz, przekazal, przekazano)
VALUES (:teczka_odnosnik, :rodzaj, :oznaczenie, :agent, 'otwarta', :otwarto, :kancelaria_xnotariusz,
  :przekazal, :przekazano);

-- name: zmien_sprawe :exec
-- Nowe oznaczenie, agent albo kancelaria sprawy, przekazane przez kierownika działu, np. po wymianie
-- agenta.
UPDATE recepcja_sprawy_wpisy
SET oznaczenie = :oznaczenie,
  agent = :agent,
  kancelaria_xnotariusz = :kancelaria_xnotariusz,
  przekazal = :przekazal,
  przekazano = :przekazano
WHERE id = :id;

-- name: zamknij_sprawe :exec
-- Zamknięcie sprawy przekazane przez kierownika działu; sprawa zostaje w spisie.
-- param zamknieto: string
UPDATE recepcja_sprawy_wpisy
SET stan = 'zamknieta',
  zamknieto = :zamknieto,
  przekazal = :przekazal,
  przekazano = :przekazano
WHERE id = :id;

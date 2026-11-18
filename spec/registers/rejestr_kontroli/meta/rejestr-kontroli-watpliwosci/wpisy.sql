-- name: watpliwosci :many
-- Wątpliwości Kontrolerów, najnowsze pierwsze; stojąca nie ma decyzji o zdjęciu.
SELECT *
FROM rejestr_kontroli_watpliwosci_wpisy
ORDER BY zgloszono DESC, id;

-- name: wpisz_watpliwosc :exec
-- Zgłoszenie wątpliwości z powodem w dniu raportu. Identyfikator (UUID) nadaje aplikacja, bo
-- wskazuje go prośba o zdjęcie.
INSERT INTO rejestr_kontroli_watpliwosci_wpisy (id, teczka_odnosnik, znacznik_odnosnik, klient_id,
  zglosil, zgloszono, powod, raport_odnosnik, wpisal, wpisano)
VALUES (:id, :teczka_odnosnik, :znacznik_odnosnik, :klient_id, :zglosil, :zgloszono, :powod,
  :raport_odnosnik, :wpisal, :wpisano);

-- name: wpisz_zdjecie_watpliwosci :exec
-- Zdjęcie wątpliwości decyzją osoby wyznaczonej z uzasadnieniem; wpisuje się raz.
-- param zdjecie_decyzja_id: string
UPDATE rejestr_kontroli_watpliwosci_wpisy
SET zdjecie_decyzja_id = :zdjecie_decyzja_id
WHERE id = :id;

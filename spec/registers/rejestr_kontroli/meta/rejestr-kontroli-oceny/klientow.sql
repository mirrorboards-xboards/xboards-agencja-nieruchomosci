-- name: oceny_ryzyka_klienta :many
-- Oceny ryzyka klienta, najnowsza pierwsza; bieżąca ma aktualna = 1.
SELECT *
FROM rejestr_kontroli_oceny_klientow
WHERE klient_id = :klient_id
ORDER BY wpisano DESC, id;

-- name: wpisz_ocene_ryzyka_klienta :exec
-- Propozycja Kontrolera do zatwierdzenia; poprzednia ocena klienta traci aktualność.
-- Identyfikator (UUID) nadaje aplikacja, bo wskazuje go prośba o zatwierdzenie.
INSERT INTO rejestr_kontroli_oceny_klientow (id, klient_id, poziom, czynniki, srodki, zaproponowal,
  raport_odnosnik, wpisal, wpisano, aktualna)
VALUES (:id, :klient_id, :poziom, :czynniki, :srodki, :zaproponowal, :raport_odnosnik, :wpisal,
  :wpisano, 1);

-- name: wpisz_zatwierdzenie_oceny :exec
-- Zatwierdzenie aktualnej oceny decyzją osoby wyznaczonej; wpisuje się raz.
-- param zatwierdzenie_decyzja_id: string
UPDATE rejestr_kontroli_oceny_klientow
SET zatwierdzenie_decyzja_id = :zatwierdzenie_decyzja_id
WHERE id = :id;

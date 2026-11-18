-- name: niemoznosci_klienta :many
-- Środki, których nie dało się zastosować wobec klienta.
SELECT *
FROM rejestr_kontroli_watpliwosci_niemoznosci
WHERE klient_id = :klient_id
ORDER BY id DESC;

-- name: wpisz_niemoznosc :exec
-- Niemożność zastosowania środka z decyzją o nienawiązaniu albo rozwiązaniu stosunków i oceną,
-- czy zawiadomić Generalnego Inspektora.
INSERT INTO rejestr_kontroli_watpliwosci_niemoznosci (klient_id, srodek, opis, decyzja_id,
  ocena_zawiadomienia, raport_odnosnik, wpisal, wpisano)
VALUES (:klient_id, :srodek, :opis, :decyzja_id, :ocena_zawiadomienia, :raport_odnosnik, :wpisal,
  :wpisano);

-- name: sprawdzenia_pep_klienta :many
-- Sprawdzenia eksponowanego stanowiska politycznego klienta i jego beneficjentów.
SELECT *
FROM rejestr_kontroli_sprawdzenia_pep
WHERE klient_id = :klient_id
ORDER BY id DESC;

-- name: wpisz_sprawdzenie_pep :exec
-- Oświadczenie i wynik sprawdzenia wykazu stanowisk; akceptację wpisuje wpisz_akceptacje_pep.
INSERT INTO rejestr_kontroli_sprawdzenia_pep (klient_id, dotyczy, osoba, oswiadczenie_sciezka,
  oswiadczenie_skrot, oswiadczenie_z_klauzula, wykaz_sprawdzono, wynik, stanowisko_do,
  zrodlo_majatku, zrodlo, raport_odnosnik, sprawdzil, sprawdzono, wpisal, wpisano, aktualne,
  poprzedni_id)
VALUES (:klient_id, :dotyczy, :osoba, :oswiadczenie_sciezka, :oswiadczenie_skrot,
  :oswiadczenie_z_klauzula, :wykaz_sprawdzono, :wynik, :stanowisko_do, :zrodlo_majatku, :zrodlo,
  :raport_odnosnik, :sprawdzil, :sprawdzono, :wpisal, :wpisano, 1, :poprzedni_id);

-- name: wpisz_akceptacje_pep :exec
-- Akceptacja właściciela biura na stosunki z osobą zajmującą eksponowane stanowisko polityczne;
-- wpisuje się raz.
-- param akceptacja_decyzja_id: string
UPDATE rejestr_kontroli_sprawdzenia_pep
SET akceptacja_decyzja_id = :akceptacja_decyzja_id
WHERE id = :id;

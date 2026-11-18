-- name: odpisy_crbr_klienta :many
-- Odpisy z Centralnego Rejestru Beneficjentów Rzeczywistych i rozbieżności, najnowsze pierwsze.
SELECT *
FROM rejestr_kontroli_sprawdzenia_crbr
WHERE klient_id = :klient_id
ORDER BY id DESC;

-- name: wpisz_odpis_crbr :exec
-- Odpis albo potwierdzenie zgłoszenia z rozbieżnością i jej wyjaśnieniem.
INSERT INTO rejestr_kontroli_sprawdzenia_crbr (klient_id, odpis_sciezka, odpis_skrot, rozbieznosc,
  rozbieznosc_opis, wyjasnienie, zrodlo, raport_odnosnik, sprawdzil, sprawdzono, wpisal, wpisano,
  aktualne, poprzedni_id)
VALUES (:klient_id, :odpis_sciezka, :odpis_skrot, :rozbieznosc, :rozbieznosc_opis, :wyjasnienie,
  :zrodlo, :raport_odnosnik, :sprawdzil, :sprawdzono, :wpisal, :wpisano, 1, :poprzedni_id);

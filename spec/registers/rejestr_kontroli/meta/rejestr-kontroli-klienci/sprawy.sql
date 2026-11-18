-- name: sprawy_klienta_kontroli :many
-- Kwalifikacje spraw klienta, najnowsze pierwsze; bieżąca kwalifikacja teczki to najnowsza.
SELECT *
FROM rejestr_kontroli_klienci_sprawy
WHERE klient_id = :klient_id
ORDER BY id DESC;

-- name: zakwalifikuj_sprawe :exec
-- Czy sprawa jest objęta ustawą, z podstawą; przy pośrednictwie najmu z czynszem, kursem euro
-- i dniem kursu.
INSERT INTO rejestr_kontroli_klienci_sprawy (klient_id, teczka_odnosnik, rodzaj_sprawy, objeta,
  podstawa, czynsz, kurs_euro, dzien_kursu, zakwalifikowano, wpisal)
VALUES (:klient_id, :teczka_odnosnik, :rodzaj_sprawy, :objeta, :podstawa, :czynsz, :kurs_euro,
  :dzien_kursu, :zakwalifikowano, :wpisal);

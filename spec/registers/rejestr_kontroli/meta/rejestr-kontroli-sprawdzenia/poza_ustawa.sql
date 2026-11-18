-- name: sprawdzenia_poza_ustawa_teczki :many
-- Sprawdzenia spoza ustawy w jednej teczce, najnowsze pierwsze.
SELECT *
FROM rejestr_kontroli_sprawdzenia_poza_ustawa
WHERE teczka_odnosnik = :teczka_odnosnik
ORDER BY id DESC;

-- name: wpisz_sprawdzenie_poza_ustawa :exec
-- Sprawdzenie tożsamości, tytułu, pełnomocnictwa albo rachunku z teczki; przy rachunku przed
-- wypłatą także wypłata. Ponowne sprawdzenie tego samego wiersza wskazuje poprzedni wpis.
INSERT INTO rejestr_kontroli_sprawdzenia_poza_ustawa (klient_id, rodzaj, teczka_odnosnik,
  wiersz_odnosnik, wersja_wiersza, sprawdzenie_odnosnik, wyplata_odnosnik, zrodlo_data, wynik, opis,
  zrodlo, raport_odnosnik, sprawdzil, sprawdzono, wpisal, wpisano, aktualne, poprzedni_id)
VALUES (:klient_id, :rodzaj, :teczka_odnosnik, :wiersz_odnosnik, :wersja_wiersza,
  :sprawdzenie_odnosnik, :wyplata_odnosnik, :zrodlo_data, :wynik, :opis, :zrodlo, :raport_odnosnik,
  :sprawdzil, :sprawdzono, :wpisal, :wpisano, 1, :poprzedni_id);

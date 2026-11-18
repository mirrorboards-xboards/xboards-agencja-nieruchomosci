-- name: dane_klienta :many
-- Wszystkie wersje danych klienta, najnowsza pierwsza; aktualna ma aktualne = 1.
SELECT *
FROM klienci_kartoteka_dane
WHERE klient_id = :klient_id
ORDER BY id DESC;

-- name: wpisz_dane_klienta :exec
-- Nowa wersja danych klienta ze źródłem w teczce, np. nowe nazwisko z aneksu. Poprzednia wersja
-- traci aktualność, ale zostaje.
INSERT INTO klienci_kartoteka_dane (klient_id, od, aktualne, imiona, nazwisko, pesel, nazwa,
  forma_prawna, krs, ceidg, nip, regon, firma, zrodlo_odnosnik, zrodlo_opis, wpisal)
VALUES (:klient_id, :od, 1, :imiona, :nazwisko, :pesel, :nazwa, :forma_prawna, :krs, :ceidg, :nip,
  :regon, :firma, :zrodlo_odnosnik, :zrodlo_opis, :wpisal);

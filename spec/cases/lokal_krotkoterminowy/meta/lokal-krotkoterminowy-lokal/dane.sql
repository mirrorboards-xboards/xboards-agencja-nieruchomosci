-- name: dane_lokalu :one
-- Dane lokalu: nieruchomość, rejestracja, łóżka, czas sprzątania, klucze, standard i instrukcja.
SELECT *
FROM lokal_krotkoterminowy_lokal_dane;

-- name: wpisz_dane_lokalu :exec
-- Dane lokalu przy przejęciu w zarząd; teczka ma jeden wiersz.
INSERT INTO lokal_krotkoterminowy_lokal_dane (id, nieruchomosc_id, rejestracja_numer,
  rejestracja_data, lozka, czas_sprzatania_min, klucze_liczba, standard_dokument_id,
  instrukcja_dokument_id)
VALUES (1, :nieruchomosc_id, :rejestracja_numer, :rejestracja_data, :lozka, :czas_sprzatania_min,
  :klucze_liczba, :standard_dokument_id, :instrukcja_dokument_id);

-- name: zmien_dane_lokalu :exec
-- Nowe dane lokalu, np. numer rejestracji albo nowa wersja standardu czy instrukcji.
UPDATE lokal_krotkoterminowy_lokal_dane
SET rejestracja_numer = :rejestracja_numer,
  rejestracja_data = :rejestracja_data,
  lozka = :lozka,
  czas_sprzatania_min = :czas_sprzatania_min,
  klucze_liczba = :klucze_liczba,
  standard_dokument_id = :standard_dokument_id,
  instrukcja_dokument_id = :instrukcja_dokument_id
WHERE id = 1;

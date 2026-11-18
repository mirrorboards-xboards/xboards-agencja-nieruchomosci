-- name: identyfikacje_klienta :many
-- Dane identyfikacyjne klienta, jego beneficjentów i osób działających za niego, najnowsze pierwsze.
SELECT *
FROM rejestr_kontroli_sprawdzenia_identyfikacje
WHERE klient_id = :klient_id
ORDER BY id DESC;

-- name: wpisz_identyfikacje :exec
-- Dane identyfikacyjne z raportu Kontrolera; ponowna identyfikacja wskazuje poprzedni wpis.
INSERT INTO rejestr_kontroli_sprawdzenia_identyfikacje (klient_id, kogo, rodzaj_podmiotu, imiona,
  nazwisko, obywatelstwo, pesel, data_urodzenia, panstwo_urodzenia, dokument_tozsamosci,
  adres_zamieszkania, firma, nip, adres_dzialalnosci, nazwa, forma_organizacyjna, adres_siedziby,
  rejestr_panstwo, rejestr_nazwa, rejestr_numer, rejestr_data, struktura_wlasnosci,
  umocowanie_dokument, zrodlo, raport_odnosnik, sprawdzil, sprawdzono, wpisal, wpisano, aktualne,
  poprzedni_id)
VALUES (:klient_id, :kogo, :rodzaj_podmiotu, :imiona, :nazwisko, :obywatelstwo, :pesel,
  :data_urodzenia, :panstwo_urodzenia, :dokument_tozsamosci, :adres_zamieszkania, :firma, :nip,
  :adres_dzialalnosci, :nazwa, :forma_organizacyjna, :adres_siedziby, :rejestr_panstwo,
  :rejestr_nazwa, :rejestr_numer, :rejestr_data, :struktura_wlasnosci, :umocowanie_dokument,
  :zrodlo, :raport_odnosnik, :sprawdzil, :sprawdzono, :wpisal, :wpisano, 1, :poprzedni_id);

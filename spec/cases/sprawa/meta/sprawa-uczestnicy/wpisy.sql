-- name: uczestnicy :many
-- Osoby i podmioty w teczce, w kolejności wpisania.
SELECT *
FROM sprawa_uczestnicy_wpisy
ORDER BY rowid;

-- name: uczestnik :one
-- Jedna osoba albo jeden podmiot z teczki.
SELECT *
FROM sprawa_uczestnicy_wpisy
WHERE id = :id;

-- name: wpisz_uczestnika :exec
-- Nowa osoba albo podmiot w teczce, z tym, skąd biuro ma jego dane. Identyfikator (UUID) nadaje
-- aplikacja, żeby od razu powiązać z nim adres, rolę albo dokument.
INSERT INTO sprawa_uczestnicy_wpisy (id, rodzaj_podmiotu, imiona, nazwisko, imie_ojca, imie_matki,
  pesel, pesel_nie_nadano, konsument, nazwa, forma_prawna, krs, ceidg, nip, regon, telefon, email,
  kanal_kontaktu, klient_biura, kartoteka_odnosnik, dane_od, dane_pozyskano)
VALUES (:id, :rodzaj_podmiotu, :imiona, :nazwisko, :imie_ojca, :imie_matki, :pesel,
  :pesel_nie_nadano, :konsument, :nazwa, :forma_prawna, :krs, :ceidg, :nip, :regon, :telefon, :email,
  :kanal_kontaktu, :klient_biura, :kartoteka_odnosnik, :dane_od, :dane_pozyskano);

-- name: zmien_dane_uczestnika :exec
-- Poprawka danych osoby albo podmiotu. Zmiana danych identyfikujących podnosi wersję uczestnika,
-- a z nią traci aktualność sprawdzenie Kontrolera (triggery w migracjach). Rodzaj podmiotu się
-- nie zmienia; osoba w miejsce podmiotu to nowy uczestnik.
UPDATE sprawa_uczestnicy_wpisy
SET imiona = :imiona,
  nazwisko = :nazwisko,
  imie_ojca = :imie_ojca,
  imie_matki = :imie_matki,
  pesel = :pesel,
  pesel_nie_nadano = :pesel_nie_nadano,
  konsument = :konsument,
  nazwa = :nazwa,
  forma_prawna = :forma_prawna,
  krs = :krs,
  ceidg = :ceidg,
  nip = :nip,
  regon = :regon
WHERE id = :id;

-- name: zmien_kontakt_uczestnika :exec
-- Telefon, adres poczty i kanał, którym osoba chce rozmawiać z biurem.
UPDATE sprawa_uczestnicy_wpisy
SET telefon = :telefon,
  email = :email,
  kanal_kontaktu = :kanal_kontaktu
WHERE id = :id;

-- name: ustal_sytuacje_rodzinna_uczestnika :exec
-- Stan cywilny i ustrój majątkowy z oświadczenia strony; dokument, na którym stoją, wskazuje
-- źródło w sprawa_dokumenty_zrodla.
UPDATE sprawa_uczestnicy_wpisy
SET stan_cywilny = :stan_cywilny,
  ustroj_majatkowy = :ustroj_majatkowy
WHERE id = :id;

-- name: polacz_uczestnika_z_kartoteka :exec
-- Odnośnik do wpisu klienta w kartotece klientów, który podaje kierownik działu.
UPDATE sprawa_uczestnicy_wpisy
SET klient_biura = :klient_biura,
  kartoteka_odnosnik = :kartoteka_odnosnik
WHERE id = :id;

-- name: odnotuj_poinformowanie_poza_teczka :exec
-- Osoba ma już informację o przetwarzaniu danych; opis mówi, skąd to wiadomo.
-- param poinformowana_poza_teczka: string
UPDATE sprawa_uczestnicy_wpisy
SET poinformowana_poza_teczka = :poinformowana_poza_teczka
WHERE id = :id;

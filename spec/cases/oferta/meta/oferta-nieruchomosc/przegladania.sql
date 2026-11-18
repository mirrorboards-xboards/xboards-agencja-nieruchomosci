-- name: przegladania_ksiag :many
-- Przeglądania ksiąg wieczystych, najnowsze pierwsze.
SELECT *
FROM oferta_nieruchomosc_przegladania
ORDER BY przegladano DESC, id;

-- name: wpisz_przegladanie_ksiegi :exec
-- Wynik przeglądania księgi z datą: właściciele z działu II, wpisy z działów III i IV
-- i rozbieżność z tym, co mówi klient. Identyfikator (UUID) nadaje aplikacja, bo wskazuje go
-- decyzja o publikacji mimo rozbieżności.
INSERT INTO oferta_nieruchomosc_przegladania (id, nieruchomosc_id, numer_kw, przegladano,
  wlasciciele_dzial_ii, wpisy_dzial_iii, wpisy_dzial_iv, rozbieznosc, rozbieznosc_opis,
  wydruk_sciezka, wydruk_skrot)
VALUES (:id, :nieruchomosc_id, :numer_kw, :przegladano, :wlasciciele_dzial_ii, :wpisy_dzial_iii,
  :wpisy_dzial_iv, :rozbieznosc, :rozbieznosc_opis, :wydruk_sciezka, :wydruk_skrot);

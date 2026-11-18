-- name: wykonawcy :many
-- Wykonawcy z aktualnymi danymi i bieżącym stanem, w kolejności wpisania.
SELECT
  wpisy.id,
  wpisy.nazwa,
  wpisy.rodzaj,
  wpisy.specjalnosc,
  dane.zakres_uslug,
  dane.umowa_do,
  dane.polisa_do,
  stany.stan,
  stany.od AS stan_od
FROM wykonawcy_spis_wpisy AS wpisy
JOIN wykonawcy_spis_dane AS dane ON dane.wykonawca_id = wpisy.id
  AND dane.aktualne = 1
JOIN wykonawcy_spis_stany AS stany ON stany.wykonawca_id = wpisy.id
  AND stany.id = (
    SELECT max(id)
    FROM wykonawcy_spis_stany
    WHERE wykonawca_id = wpisy.id
  )
ORDER BY wpisy.rowid;

-- name: wpisz_wykonawce :exec
-- Nowy wykonawca po decyzji o przyjęciu i podpisaniu umowy: wpis, pierwsza wersja danych i stan
-- „zatwierdzony” z tą samą decyzją. Identyfikator (UUID) nadaje aplikacja.
-- param wpisano: string
INSERT INTO wykonawcy_spis_wpisy (id, nazwa, rodzaj, specjalnosc, decyzja_id, wpisano)
VALUES (:id, :nazwa, :rodzaj, :specjalnosc, :decyzja_id, :wpisano);
INSERT INTO wykonawcy_spis_dane (wykonawca_id, aktualne, firma, nip, krs, ceidg, adres, zakres_uslug,
  srodki_wlasne, umowa_od, umowa_do, umowa_podpisal, umowa_sciezka, umowa_skrot, polisa_do,
  polisa_sciezka, polisa_skrot, decyzja_id, wpisano)
VALUES (:id, 1, :firma, :nip, :krs, :ceidg, :adres, :zakres_uslug, :srodki_wlasne, :umowa_od,
  :umowa_do, :umowa_podpisal, :umowa_sciezka, :umowa_skrot, :polisa_do, :polisa_sciezka,
  :polisa_skrot, :decyzja_id, :wpisano);
INSERT INTO wykonawcy_spis_stany (wykonawca_id, stan, od, decyzja_id)
VALUES (:id, 'zatwierdzony', date(:wpisano), :decyzja_id);

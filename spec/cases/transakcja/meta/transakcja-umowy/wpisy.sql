-- name: umowy_transakcji :many
-- Umowy stron w kolejności wpisania.
SELECT *
FROM transakcja_umowy_wpisy
ORDER BY rowid;

-- name: przygotuj_umowe_transakcji :exec
-- Projekt umowy rezerwacyjnej, przedwstępnej, warunkowej albo przenoszącej; identyfikator (UUID)
-- nadaje aplikacja, bo wskazują go strony, płatności i analizy.
INSERT INTO transakcja_umowy_wpisy (id, rodzaj, forma, oparta_na_id, kancelaria_id,
  projekt_dokument_id, cena, termin_przyrzeczonej, termin_zaplaty, termin_wydania, warunki, stan)
VALUES (:id, :rodzaj, :forma, :oparta_na_id, :kancelaria_id, :projekt_dokument_id, :cena,
  :termin_przyrzeczonej, :termin_zaplaty, :termin_wydania, :warunki, 'projekt');

-- name: zmien_projekt_umowy_transakcji :exec
-- Nowa wersja projektu: inny dokument projektu albo inne warunki. Nowy dokument wymaga nowej
-- analizy, zanim strony podpiszą umowę.
UPDATE transakcja_umowy_wpisy
SET kancelaria_id = :kancelaria_id,
  projekt_dokument_id = :projekt_dokument_id,
  cena = :cena,
  termin_przyrzeczonej = :termin_przyrzeczonej,
  termin_zaplaty = :termin_zaplaty,
  termin_wydania = :termin_wydania,
  warunki = :warunki
WHERE id = :id;

-- name: odnotuj_zawarcie_umowy_transakcji :exec
-- Strony podpisały umowę: podpisany egzemplarz albo wypis aktu, chwila i przy akcie numer
-- repertorium.
-- param dokument_id: string
-- param zawarto: string
UPDATE transakcja_umowy_wpisy
SET stan = 'zawarta',
  dokument_id = :dokument_id,
  zawarto = :zawarto,
  repertorium = :repertorium
WHERE id = :id;

-- name: odnotuj_ze_umowa_transakcji_nie_doszla :exec
-- Projekt, którego strony nie podpiszą, np. bo bank odmówił kredytu.
-- param koniec_powod: string
UPDATE transakcja_umowy_wpisy
SET stan = 'nie_doszla',
  koniec_powod = :koniec_powod
WHERE id = :id;

-- name: rozwiaz_umowe_transakcji :exec
-- Zawarta umowa przestaje wiązać, np. po odstąpieniu z zadatkiem (art. 394 KC).
-- param koniec_powod: string
UPDATE transakcja_umowy_wpisy
SET stan = 'rozwiazana',
  koniec_powod = :koniec_powod
WHERE id = :id;

-- name: odnotuj_wniosek_do_ksiegi :exec
-- Notariusz złożył wniosek o wpis kupującego do księgi wieczystej (art. 92 § 4 PoN).
-- param wniosek_kw_zlozono: string
UPDATE transakcja_umowy_wpisy
SET wniosek_kw_zlozono = :wniosek_kw_zlozono
WHERE id = :id;

-- name: odnotuj_wpis_do_ksiegi :exec
-- Sąd wpisał kupującego do księgi; Koordynator informuje o tym strony.
-- param wpis_kw: string
UPDATE transakcja_umowy_wpisy
SET wpis_kw = :wpis_kw
WHERE id = :id;

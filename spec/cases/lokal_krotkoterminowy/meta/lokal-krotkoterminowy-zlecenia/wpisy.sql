-- name: zlecenia :many
-- Zlecenia w kolejności okna pracy.
SELECT *
FROM lokal_krotkoterminowy_zlecenia_wpisy
ORDER BY okno_od, id;

-- name: zlecenia_otwarte :many
-- Zlecenia zlecone i wstrzymane w kolejności okna pracy.
SELECT *
FROM lokal_krotkoterminowy_zlecenia_wpisy
WHERE stan IN ('zlecone', 'wstrzymane')
ORDER BY okno_od, id;

-- name: zlec_prace :exec
-- Zlecenie dla wykonawcy z rejestru albo spoza niego, z oknem i ceną; gdy ma powód zgody,
-- wskazuje decyzję człowieka i ewentualnie zgodę właściciela. Identyfikator (UUID) nadaje
-- aplikacja, bo wskazują go decyzja, zgoda, noce, dostępy i pozycje zestawień.
INSERT INTO lokal_krotkoterminowy_zlecenia_wpisy (id, rodzaj, przyczyna, pobyt_id, zgloszenie_id,
  wykonawca_odnosnik, spoza_rejestru, cennik_wersja_odnosnik, poza_standardem, okno_od, okno_do, cena,
  cena_zrodlo, zgoda_id, decyzja_id, zlecil, zlecono, stan)
VALUES (:id, :rodzaj, :przyczyna, :pobyt_id, :zgloszenie_id, :wykonawca_odnosnik, :spoza_rejestru,
  :cennik_wersja_odnosnik, :poza_standardem, :okno_od, :okno_do, :cena, :cena_zrodlo, :zgoda_id,
  :decyzja_id, :zlecil, :zlecono, 'zlecone');

-- name: zmien_stan_zlecenia :exec
-- Wstrzymanie, wznowienie albo anulowanie zlecenia.
-- param stan: "zlecone" | "wstrzymane" | "anulowane"
UPDATE lokal_krotkoterminowy_zlecenia_wpisy
SET stan = :stan
WHERE id = :id;

-- name: potwierdz_wykonanie_zlecenia :exec
-- Raport wykonawcy ze zdjęciami porównany z listą kontrolną i potwierdzenie wykonania.
-- param raport_sciezka: string
-- param raport_skrot: string
-- param potwierdzil: string
-- param potwierdzono: string
UPDATE lokal_krotkoterminowy_zlecenia_wpisy
SET stan = 'potwierdzone',
  raport_sciezka = :raport_sciezka,
  raport_skrot = :raport_skrot,
  potwierdzil = :potwierdzil,
  potwierdzono = :potwierdzono
WHERE id = :id;

-- name: wpisz_koszt_zlecenia :exec
-- Koszt po potwierdzeniu wykonania; inny niż cena wskazuje decyzję człowieka.
-- param koszt: number
UPDATE lokal_krotkoterminowy_zlecenia_wpisy
SET koszt = :koszt,
  koszt_decyzja_id = :koszt_decyzja_id
WHERE id = :id;

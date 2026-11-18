-- name: konce_najmu :many
-- Koniec najmu: sposób, pismo, dzień końca, żądanie opróżnienia i opróżnienie.
SELECT *
FROM najem_umowy_konce
ORDER BY rowid;

-- name: wpisz_koniec_najmu :exec
-- Jak najem się kończy: upływ czasu, wypowiedzenie (pismo wynajmującego jeszcze przed wysłaniem
-- albo pismo najemcy), porozumienie albo śmierć najemcy; identyfikator (UUID) nadaje aplikacja.
INSERT INTO najem_umowy_konce (id, najem_id, sposob, przyczyna, wysylka_id, wplyw_id, koniec_najmu)
VALUES (:id, :najem_id, :sposob, :przyczyna, :wysylka_id, :wplyw_id, :koniec_najmu);

-- name: odnotuj_doreczenie_konca_najmu :exec
-- Pismo o końcu najmu doręczono.
-- param doreczono: string
UPDATE najem_umowy_konce
SET doreczono = :doreczono
WHERE id = :id;

-- name: odnotuj_decyzje_po_smierci_najemcy :exec
-- Człowiek ocenił, kto wstąpił w najem; gdy nikt, dzień wygaśnięcia najmu.
-- param decyzja_id: string
UPDATE najem_umowy_konce
SET decyzja_id = :decyzja_id,
  koniec_najmu = :koniec_najmu
WHERE id = :id;

-- name: wskaz_zadanie_oproznienia :exec
-- Żądanie opróżnienia lokalu przy najmie okazjonalnym albo instytucjonalnym, jeszcze przed wysłaniem.
-- param zadanie_wysylka_id: string
UPDATE najem_umowy_konce
SET zadanie_wysylka_id = :zadanie_wysylka_id
WHERE id = :id;

-- name: odnotuj_doreczenie_zadania_oproznienia :exec
-- Żądanie doręczono; termin opróżnienia z żądania.
-- param zadanie_doreczono: string
-- param zadanie_termin: string
UPDATE najem_umowy_konce
SET zadanie_doreczono = :zadanie_doreczono,
  zadanie_termin = :zadanie_termin
WHERE id = :id;

-- name: odnotuj_oproznienie_lokalu :exec
-- Najemca oddał klucze i opuścił lokal; od tego dnia biegnie miesiąc na zwrot kaucji.
-- param oproznienie: string
UPDATE najem_umowy_konce
SET oproznienie = :oproznienie
WHERE id = :id;

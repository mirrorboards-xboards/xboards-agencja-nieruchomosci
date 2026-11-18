-- name: oferty_w_kanalach :many
-- Oferty lokalu w kanałach sprzedaży.
SELECT *
FROM lokal_krotkoterminowy_kalendarz_kanaly
ORDER BY kanal, id;

-- name: przygotuj_oferte_w_kanale :exec
-- Projekt oferty w kanale na koncie biura albo właściciela. Nowy kanał spoza umowy wskazuje zgodę
-- właściciela. Identyfikator (UUID) nadaje aplikacja, bo wskazuje go decyzja o nowym kanale.
INSERT INTO lokal_krotkoterminowy_kalendarz_kanaly (id, kanal, konto, nazwa_konta, rachunek_id,
  zgoda_id, stan)
VALUES (:id, :kanal, :konto, :nazwa_konta, :rachunek_id, :zgoda_id, 'projekt');

-- name: opublikuj_oferte_w_kanale :exec
-- Oferta opublikowana z numerem w kanale; nowy kanał wskazuje zgodę człowieka.
-- param numer_oferty: string
-- param opublikowano: string
UPDATE lokal_krotkoterminowy_kalendarz_kanaly
SET stan = 'aktywna',
  numer_oferty = :numer_oferty,
  opublikowano = :opublikowano,
  nowy_kanal_decyzja_id = :nowy_kanal_decyzja_id
WHERE id = :id;

-- name: zmien_stan_oferty_w_kanale :exec
-- Zawieszenie przez kanał, wznowienie albo zamknięcie oferty.
-- param stan: "aktywna" | "zawieszona" | "zamknieta"
UPDATE lokal_krotkoterminowy_kalendarz_kanaly
SET stan = :stan
WHERE id = :id;

-- name: wskaz_rachunek_oferty_w_kanale :exec
-- Rachunek, na który kanał wypłaca przychód; zmienia go człowiek po sprawdzeniu Kontrolera.
UPDATE lokal_krotkoterminowy_kalendarz_kanaly
SET rachunek_id = :rachunek_id
WHERE id = :id;

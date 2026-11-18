-- name: obszary :many
-- Wszystkie obszary; najpierw niezniesione, według nazwy.
SELECT *
FROM kierownik_najmu_obszary_wpisy
ORDER BY zniesiono IS NOT NULL, nazwa, id;

-- name: obszary_koordynatora :many
-- Niezniesione obszary jednego Koordynatora operacji.
SELECT *
FROM kierownik_najmu_obszary_wpisy
WHERE koordynator_id = :koordynator_id
  AND zniesiono IS NULL
ORDER BY nazwa, id;

-- name: zaproponuj_obszar :exec
-- Nowy obszar z granicami, które kierownik proponuje człowiekowi.
INSERT INTO kierownik_najmu_obszary_wpisy (nazwa, granice, zaproponowano)
VALUES (:nazwa, :granice, :zaproponowano);

-- name: zmien_propozycje_obszaru :exec
-- Kierownik poprawia granice, których człowiek jeszcze nie zatwierdził.
UPDATE kierownik_najmu_obszary_wpisy
SET nazwa = :nazwa,
  granice = :granice
WHERE id = :id;

-- name: zatwierdz_obszar :exec
-- Człowiek zatwierdził granice podpisaną wiadomością.
-- param zatwierdzil: string
-- param podpis: string
-- param zatwierdzono: string
UPDATE kierownik_najmu_obszary_wpisy
SET zatwierdzil = :zatwierdzil,
  podpis = :podpis,
  zatwierdzono = :zatwierdzono
WHERE id = :id;

-- name: wskaz_koordynatora_obszaru :exec
-- Koordynator operacji obszaru, także następca po wymianie poprzedniego.
-- param koordynator_id: string
UPDATE kierownik_najmu_obszary_wpisy
SET koordynator_id = :koordynator_id
WHERE id = :id;

-- name: znies_obszar :exec
-- Obszar przestaje istnieć, np. bo człowiek zatwierdził nowe granice jako nowy obszar.
-- param zniesiono: string
UPDATE kierownik_najmu_obszary_wpisy
SET zniesiono = :zniesiono
WHERE id = :id;

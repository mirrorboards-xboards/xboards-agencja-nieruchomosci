-- name: pary :many
-- Wszystkie pary obu stron, najnowsze pierwsze.
SELECT *
FROM kierownik_posrednictwa_obie_strony_pary
ORDER BY stwierdzono DESC, id;

-- name: pary_do_wyjasnienia :many
-- Pary, w których nie wiadomo jeszcze, czy umowy obu stron pozwalają biuru działać dla drugiej
-- strony, od najstarszej.
SELECT *
FROM kierownik_posrednictwa_obie_strony_pary
WHERE umowy_pozwalaja = 'do_wyjasnienia'
ORDER BY stwierdzono, id;

-- name: zapisz_pare :exec
-- Oferta i poszukiwanie spotkały się w jednej transakcji.
INSERT INTO kierownik_posrednictwa_obie_strony_pary (teczka_oferty_odnosnik,
  teczka_poszukiwania_odnosnik, stwierdzono, umowy_pozwalaja)
VALUES (:teczka_oferty_odnosnik, :teczka_poszukiwania_odnosnik, :stwierdzono, 'do_wyjasnienia');

-- name: ocen_umowy_pary :exec
-- Czy umowy pośrednictwa obu stron pozwalają biuru działać dla drugiej strony; „tak” wskazuje obie
-- umowy.
UPDATE kierownik_posrednictwa_obie_strony_pary
SET umowy_pozwalaja = :umowy_pozwalaja,
  umowa_oferty_odnosnik = :umowa_oferty_odnosnik,
  umowa_poszukiwania_odnosnik = :umowa_poszukiwania_odnosnik
WHERE id = :id;

-- name: odnotuj_przekazanie_pary :exec
-- Przyjęta oferta przekazana Kierownikowi transakcji; bez zgody obu umów tylko za zgodą człowieka.
-- param przekazano: string
UPDATE kierownik_posrednictwa_obie_strony_pary
SET przekazano = :przekazano,
  zatwierdzil = :zatwierdzil,
  podpis = :podpis,
  zatwierdzono = :zatwierdzono
WHERE id = :id;

-- name: odnotuj_teczke_transakcji :exec
-- Kierownik transakcji założył teczkę transakcji dla pary.
-- param transakcja_odnosnik: string
UPDATE kierownik_posrednictwa_obie_strony_pary
SET transakcja_odnosnik = :transakcja_odnosnik
WHERE id = :id;

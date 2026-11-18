-- name: wersje_standardu :many
-- Wszystkie wersje jednego standardu działu, od najnowszej.
SELECT *
FROM kierownik_dzialu_standardy_wersje
WHERE standard = :standard
ORDER BY numer DESC;

-- name: obowiazujace_standardy :many
-- Najnowsza zatwierdzona wersja każdego standardu działu; to one obowiązują.
SELECT *
FROM kierownik_dzialu_standardy_wersje AS wersja
WHERE wersja.rozstrzygniecie = 'zatwierdzona'
  AND NOT EXISTS (
    SELECT 1
    FROM kierownik_dzialu_standardy_wersje AS nowsza
    WHERE nowsza.standard = wersja.standard
      AND nowsza.rozstrzygniecie = 'zatwierdzona'
      AND nowsza.numer > wersja.numer
  )
ORDER BY wersja.standard;

-- name: zaproponuj_standard :exec
-- Nowa wersja standardu z plikiem w XGit pokoju pamięci, czekająca na rozstrzygnięcie człowieka.
INSERT INTO kierownik_dzialu_standardy_wersje (standard, numer, sciezka, skrot, opis_zmian, zaproponowano)
VALUES (:standard, :numer, :sciezka, :skrot, :opis_zmian, :zaproponowano);

-- name: rozstrzygnij_standard :exec
-- Zatwierdzenie albo odrzucenie wersji przez właściciela kierownika albo, w Kontroli, właściciela
-- biura, z podpisem obejmującym skrót pliku.
-- param rozstrzygniecie: "zatwierdzona" | "odrzucona"
-- param podpisal: string
-- param podpisal_jako: "wlasciciel" | "wlasciciel_biura"
-- param podpis: string
-- param podpisano: string
UPDATE kierownik_dzialu_standardy_wersje
SET rozstrzygniecie = :rozstrzygniecie,
  podpisal = :podpisal,
  podpisal_jako = :podpisal_jako,
  podpis = :podpis,
  podpisano = :podpisano
WHERE id = :id;

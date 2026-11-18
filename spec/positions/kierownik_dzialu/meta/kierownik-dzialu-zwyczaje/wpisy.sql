-- name: zwyczaje_kontrahenta :many
-- Niewykreślone zwyczaje jednego kontrahenta; pewne pierwsze.
SELECT *
FROM kierownik_dzialu_zwyczaje_wpisy
WHERE kontrahent = :kontrahent
  AND wykreslono IS NULL
ORDER BY pewnosc = 'niepewny', pewnosc = 'prawdopodobny', temat, id;

-- name: zwyczaje :many
-- Wszystkie zwyczaje, także wykreślone, według rodzaju kontrahenta, kontrahenta i tematu.
SELECT *
FROM kierownik_dzialu_zwyczaje_wpisy
ORDER BY kontrahent_rodzaj, kontrahent, temat, id;

-- name: zwyczaje_od_zrodla :many
-- Niewykreślone zwyczaje od jednego źródła; po jego nieprawdzie kierownik ocenia je na nowo.
SELECT *
FROM kierownik_dzialu_zwyczaje_wpisy
WHERE zrodlo_id = :zrodlo_id
  AND wykreslono IS NULL
ORDER BY ustalono, id;

-- name: wpisz_zwyczaj :exec
-- Zwyczaj zauważony przez kierownika, jeszcze bez potwierdzenia: prawdopodobny albo niepewny.
-- param pewnosc: "prawdopodobny" | "niepewny"
INSERT INTO kierownik_dzialu_zwyczaje_wpisy (kontrahent_rodzaj, kontrahent, temat, opis, zrodlo_id,
  zrodlo_odnosnik, ustalono, pewnosc)
VALUES (:kontrahent_rodzaj, :kontrahent, :temat, :opis, :zrodlo_id, :zrodlo_odnosnik, :ustalono,
  :pewnosc);

-- name: potwierdz_zwyczaj :exec
-- Zwyczaj potwierdził regulamin albo kontrahent (odnośnik) albo człowiek biura podpisaną
-- wiadomością (potwierdził i podpis), więc zwyczaj jest pewny.
-- param potwierdzenie: "regulamin" | "kontrahent" | "czlowiek_biura"
-- param potwierdzono: string
UPDATE kierownik_dzialu_zwyczaje_wpisy
SET pewnosc = 'pewny',
  potwierdzenie = :potwierdzenie,
  potwierdzenie_odnosnik = :potwierdzenie_odnosnik,
  potwierdzil = :potwierdzil,
  podpis = :podpis,
  potwierdzono = :potwierdzono
WHERE id = :id;

-- name: zakwestionuj_zwyczaj :exec
-- Zwyczaj okazał się wątpliwy, np. dwa raporty mówią co innego albo źródło podało nieprawdę, więc
-- traci potwierdzenie i dostaje nową pewność.
-- param pewnosc: "prawdopodobny" | "niepewny"
UPDATE kierownik_dzialu_zwyczaje_wpisy
SET pewnosc = :pewnosc,
  potwierdzenie = NULL,
  potwierdzenie_odnosnik = NULL,
  potwierdzil = NULL,
  podpis = NULL,
  potwierdzono = NULL
WHERE id = :id;

-- name: wykresl_zwyczaj :exec
-- Zwyczaj okazał się nieprawdziwy albo przestał obowiązywać; zostaje w pamięci jako wykreślony.
-- param wykreslono: string
-- param powod_wykreslenia: string
UPDATE kierownik_dzialu_zwyczaje_wpisy
SET wykreslono = :wykreslono,
  powod_wykreslenia = :powod_wykreslenia
WHERE id = :id;

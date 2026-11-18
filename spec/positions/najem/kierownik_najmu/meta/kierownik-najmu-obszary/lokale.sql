-- name: lokale_obszaru :many
-- Lokale, które teraz należą do jednego obszaru.
SELECT *
FROM kierownik_najmu_obszary_lokale
WHERE obszar_id = :obszar_id
  AND do_dnia IS NULL
ORDER BY od, id;

-- name: obszar_lokalu :many
-- Obszar, do którego teraz należy lokal, z jego Koordynatorem operacji.
SELECT obszar.*
FROM kierownik_najmu_obszary_lokale AS lokal
JOIN kierownik_najmu_obszary_wpisy AS obszar ON obszar.id = lokal.obszar_id
WHERE lokal.teczka_odnosnik = :teczka_odnosnik
  AND lokal.do_dnia IS NULL;

-- name: lokale_bez_koordynatora :many
-- Lokale w obszarach, które nie mają Koordynatora operacji; każdy lokal powinien go mieć.
SELECT lokal.*
FROM kierownik_najmu_obszary_lokale AS lokal
JOIN kierownik_najmu_obszary_wpisy AS obszar ON obszar.id = lokal.obszar_id
WHERE lokal.do_dnia IS NULL
  AND obszar.koordynator_id IS NULL
ORDER BY lokal.od, lokal.id;

-- name: przypisz_lokal_do_obszaru :exec
-- Lokal należy od dziś do obszaru; poprzednią przynależność trzeba najpierw zakończyć.
INSERT INTO kierownik_najmu_obszary_lokale (obszar_id, teczka_odnosnik, od)
VALUES (:obszar_id, :teczka_odnosnik, :od);

-- name: zakoncz_przynaleznosc_lokalu :exec
-- Lokal przestaje należeć do obszaru, np. bo przechodzi do innego albo biuro przestało nim zarządzać.
-- param do_dnia: string
UPDATE kierownik_najmu_obszary_lokale
SET do_dnia = :do_dnia
WHERE id = :id;

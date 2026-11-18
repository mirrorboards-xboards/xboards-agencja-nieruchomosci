-- name: polisy :many
-- Wszystkie polisy, najnowsze okresy pierwsze.
SELECT *
FROM kierownik_biura_polisy_wpisy
ORDER BY okres_do DESC, id;

-- name: polisa_w_dniu :many
-- Polisy danego rodzaju aktualne w jednym dniu (RRRR-MM-DD), np. w dniu zawarcia umowy.
-- param dzien: string
SELECT *
FROM kierownik_biura_polisy_wpisy
WHERE rodzaj = :rodzaj
  AND okres_od <= :dzien
  AND okres_do >= :dzien
ORDER BY okres_od, id;

-- name: polisy_do_przypomnienia :many
-- Polisy, które kończą się nie później niż `do_dnia`, a kierownik jeszcze o nich nie przypomniał.
-- param do_dnia: string
SELECT *
FROM kierownik_biura_polisy_wpisy
WHERE przypomniano IS NULL
  AND okres_do <= :do_dnia
ORDER BY okres_do, id;

-- name: zapisz_polise :exec
-- Nowa polisa albo zmienione dane polisy, z odwołaniem do kopii w dokumentach biura.
INSERT INTO kierownik_biura_polisy_wpisy (rodzaj, numer, ubezpieczyciel, okres_od, okres_do,
  dokument_odnosnik, zapisano)
VALUES (:rodzaj, :numer, :ubezpieczyciel, :okres_od, :okres_do, :dokument_odnosnik, :zapisano);

-- name: odnotuj_przypomnienie_o_polisie :exec
-- Kierownik przypomniał właścicielowi biura o końcu polisy.
-- param przypomniano: string
UPDATE kierownik_biura_polisy_wpisy
SET przypomniano = :przypomniano
WHERE id = :id;

-- name: odnotuj_zawiadomienie_o_polisie :exec
-- Kierownik zawiadomił kierownika działu o nowej polisie.
-- param zawiadomiono: string
-- param zawiadomiono_kogo: "kierownik_posrednictwa" | "kierownik_najmu"
UPDATE kierownik_biura_polisy_wpisy
SET zawiadomiono = :zawiadomiono,
  zawiadomiono_kogo = :zawiadomiono_kogo
WHERE id = :id;

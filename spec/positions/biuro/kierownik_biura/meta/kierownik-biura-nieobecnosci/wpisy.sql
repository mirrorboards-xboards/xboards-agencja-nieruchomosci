-- name: nieobecnosci_w_dniu :many
-- Nieodwołane nieobecności, które obejmują jeden dzień (RRRR-MM-DD).
-- param dzien: string
SELECT *
FROM kierownik_biura_nieobecnosci_wpisy
WHERE odwolano IS NULL
  AND od_dnia <= :dzien
  AND do_dnia >= :dzien
ORDER BY czlowiek, id;

-- name: nieobecnosci_od :many
-- Nieobecności, które trwały albo trwają od jednego dnia, także odwołane, do sprawozdania.
-- param od: string
SELECT *
FROM kierownik_biura_nieobecnosci_wpisy
WHERE do_dnia >= :od
ORDER BY od_dnia, id;

-- name: zapisz_nieobecnosc :exec
-- Nieobecność ze zastępcą z podpisanej wiadomości człowieka albo właściciela biura.
-- param podpisal_jako: "czlowiek" | "wlasciciel_biura"
INSERT INTO kierownik_biura_nieobecnosci_wpisy (czlowiek, od_dnia, do_dnia, zastepca, podpisal,
  podpisal_jako, podpis, wiadomosc_odnosnik, zapisano)
VALUES (:czlowiek, :od_dnia, :do_dnia, :zastepca, :podpisal, :podpisal_jako, :podpis,
  :wiadomosc_odnosnik, :zapisano);

-- name: odwolaj_nieobecnosc :exec
-- Podpisane odwołanie nieobecności.
-- param odwolal: string
-- param odwolanie_podpis: string
-- param odwolano: string
UPDATE kierownik_biura_nieobecnosci_wpisy
SET odwolal = :odwolal,
  odwolanie_podpis = :odwolanie_podpis,
  odwolano = :odwolano
WHERE id = :id;

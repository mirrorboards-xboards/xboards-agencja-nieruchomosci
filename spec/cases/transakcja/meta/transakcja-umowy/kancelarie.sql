-- name: kancelarie :many
-- Kancelarie transakcji w kolejności wpisania; bieżąca nie ma daty końca.
SELECT *
FROM transakcja_umowy_kancelarie
ORDER BY od, id;

-- name: wpisz_kancelarie :exec
-- Kancelaria, którą wybrały strony; identyfikator (UUID) nadaje aplikacja, bo wskazuje go decyzja
-- o otwarciu pokoju wymiany.
INSERT INTO transakcja_umowy_kancelarie (id, nazwa, adres, notariusz, xnotariusz, od)
VALUES (:id, :nazwa, :adres, :notariusz, :xnotariusz, :od);

-- name: otworz_pokoj_wymiany :exec
-- Pokój wymiany z kancelarią w XNotariuszu, otwarty za zatwierdzoną decyzją inna, która wskazuje
-- tę kancelarię.
-- param otwarcie_decyzja_id: string
-- param pokoj_odnosnik: string
UPDATE transakcja_umowy_kancelarie
SET otwarcie_decyzja_id = :otwarcie_decyzja_id,
  pokoj_odnosnik = :pokoj_odnosnik
WHERE id = :id;

-- name: zakoncz_kancelarie :exec
-- Strony zmieniają kancelarię: bieżąca dostaje datę końca i powód, a nowa to nowy wpis.
-- param do: string
-- param powod_zmiany: string
UPDATE transakcja_umowy_kancelarie
SET do = :do,
  powod_zmiany = :powod_zmiany
WHERE id = :id;

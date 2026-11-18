-- name: decyzje :many
-- Wszystkie prośby o decyzję i rozstrzygnięcia sprawy, najnowsze pierwsze.
SELECT *
FROM sprawa_decyzje_wpisy
ORDER BY zlozono DESC, id;

-- name: decyzje_czekajace :many
-- Prośby, które czekają na rozstrzygnięcie, najstarsze pierwsze; z nich Kierownik biura układa
-- listę dla każdego człowieka.
SELECT *
FROM sprawa_decyzje_wpisy
WHERE stan = 'czeka'
ORDER BY zlozono, id;

-- name: decyzje_zatwierdzone :many
-- Obowiązujące zgody: decyzje rozstrzygnięte zgodą albo zgodą z warunkiem, najnowsze pierwsze.
SELECT *
FROM sprawa_decyzje_zatwierdzone
ORDER BY podpisano DESC, id;

-- name: decyzje_wiersza :many
-- Decyzje, które wskazują jeden wiersz teczki, najnowsze pierwsze.
SELECT *
FROM sprawa_decyzje_wpisy
WHERE dotyczy_tabela = :dotyczy_tabela
  AND dotyczy_id = :dotyczy_id
ORDER BY zlozono DESC, id;

-- name: popros_o_decyzje :exec
-- Prośba agenta o decyzję człowieka; identyfikator (UUID) nadaje aplikacja, żeby wiersz, który
-- czeka na decyzję, mógł go od razu wskazać.
INSERT INTO sprawa_decyzje_wpisy (id, rodzaj, prosi, prosi_stanowisko, decyduje, do_kiedy, zlozono,
  tresc, dotyczy_tabela, dotyczy_id, dotyczy_skrot, zastepuje_decyzja_id, stan)
VALUES (:id, :rodzaj, :prosi, :prosi_stanowisko, :decyduje, :do_kiedy, :zlozono, :tresc,
  :dotyczy_tabela, :dotyczy_id, :dotyczy_skrot, :zastepuje_decyzja_id, 'czeka');

-- name: rozstrzygnij_decyzje :exec
-- Rozstrzygnięcie prośby przez człowieka albo jego zastępcę, z podpisem kluczem xkeys.
-- param rozstrzygniecie: "zgoda" | "odmowa" | "zgoda_z_warunkiem"
-- param podpisal: string
-- param podpis: string
-- param podpisano: string
UPDATE sprawa_decyzje_wpisy
SET stan = 'rozstrzygnieta',
  rozstrzygniecie = :rozstrzygniecie,
  warunek = :warunek,
  uzasadnienie = :uzasadnienie,
  podpisal = :podpisal,
  za_kogo = :za_kogo,
  podpis = :podpis,
  podpisano = :podpisano
WHERE id = :id;

-- name: uznaj_prosbe_za_zbedna :exec
-- Prośba, która przestała być potrzebna, np. bo sprawa się zmieniła; zostaje z powodem.
-- param powod_zbednosci: string
UPDATE sprawa_decyzje_wpisy
SET stan = 'zbedna',
  powod_zbednosci = :powod_zbednosci
WHERE id = :id;

-- name: karty_otwarte :many
-- Karty, które czekają na propozycję albo na pośrednika, od najstarszej.
SELECT *
FROM kierownik_posrednictwa_przydzial_karty
WHERE stan IN ('czeka', 'zaproponowana')
ORDER BY otrzymano, id;

-- name: przyjmij_karte :exec
-- Karta nowej sprawy od Kierownika biura.
INSERT INTO kierownik_posrednictwa_przydzial_karty (karta_odnosnik, otrzymano, kontakt, stan)
VALUES (:karta_odnosnik, :otrzymano, :kontakt, 'czeka');

-- name: zaproponuj_posrednika :exec
-- Propozycja pośrednika według reguły albo, za zgodą człowieka, poza regułami.
-- param posrednik: string
-- param zaproponowano: string
UPDATE kierownik_posrednictwa_przydzial_karty
SET stan = 'zaproponowana',
  posrednik = :posrednik,
  regula_id = :regula_id,
  zaproponowano = :zaproponowano,
  zatwierdzil = :zatwierdzil,
  podpis = :podpis,
  zatwierdzono = :zatwierdzono
WHERE id = :id;

-- name: cofnij_propozycje :exec
-- Pośrednik nie przyjął sprawy, więc karta czeka na nową propozycję.
UPDATE kierownik_posrednictwa_przydzial_karty
SET stan = 'czeka',
  posrednik = NULL,
  regula_id = NULL,
  zaproponowano = NULL,
  zatwierdzil = NULL,
  podpis = NULL,
  zatwierdzono = NULL
WHERE id = :id;

-- name: odnotuj_zalozenie_teczki :exec
-- Pośrednik przyjął sprawę i założył teczkę; kontakt trafił do teczki, więc znika z karty.
-- param teczka_odnosnik: string
-- param rozstrzygnieto: string
UPDATE kierownik_posrednictwa_przydzial_karty
SET stan = 'przyjeta',
  teczka_odnosnik = :teczka_odnosnik,
  rozstrzygnieto = :rozstrzygnieto,
  kontakt = NULL
WHERE id = :id;

-- name: odrzuc_karte :exec
-- Karta odrzucona z powodem; kontakt znika z karty.
-- param powod: string
-- param rozstrzygnieto: string
UPDATE kierownik_posrednictwa_przydzial_karty
SET stan = 'odrzucona',
  posrednik = NULL,
  regula_id = NULL,
  zaproponowano = NULL,
  powod = :powod,
  rozstrzygnieto = :rozstrzygnieto,
  kontakt = NULL
WHERE id = :id;

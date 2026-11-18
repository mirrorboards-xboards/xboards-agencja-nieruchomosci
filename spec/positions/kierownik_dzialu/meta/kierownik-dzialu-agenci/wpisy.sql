-- name: agenci :many
-- Wszyscy agenci powołani przez kierownika; najpierw pracujący, potem śpiący i wymienieni.
SELECT *
FROM kierownik_dzialu_agenci_wpisy
ORDER BY stan = 'wymieniony', stan = 'spi', powolano, id;

-- name: agenci_teczki :many
-- Agenci jednej teczki, także wymienieni, od pierwszego powołanego.
SELECT *
FROM kierownik_dzialu_agenci_wpisy
WHERE teczka_odnosnik = :teczka_odnosnik
ORDER BY powolano, id;

-- name: agenci_klienta :many
-- Kontrolerzy jednego klienta, także wymienieni, od pierwszego powołanego.
SELECT *
FROM kierownik_dzialu_agenci_wpisy
WHERE klient_odnosnik = :klient_odnosnik
ORDER BY powolano, id;

-- name: powolaj_agenta :exec
-- Nowy agent do teczki, a Koordynator operacji do obszaru, a Kontroler do klienta; identyfikator
-- (UUID) nadaje aplikacja, żeby raporty, wpuszczenie i wymiana mogły go od razu wskazać.
-- Zatwierdzenie człowieka przychodzi z powołaniem w okresie wdrożenia.
INSERT INTO kierownik_dzialu_agenci_wpisy (id, stanowisko, tozsamosc, wlasciciel, teczka_odnosnik,
  klient_odnosnik, powolano, stan, zatwierdzil, podpis, zatwierdzono)
VALUES (:id, :stanowisko, :tozsamosc, :wlasciciel, :teczka_odnosnik, :klient_odnosnik, :powolano,
  'pracuje', :zatwierdzil, :podpis, :zatwierdzono);

-- name: zatwierdz_powolanie :exec
-- Człowiek zatwierdził powołanie podpisaną wiadomością, gdy przyszła po powołaniu.
-- param zatwierdzil: string
-- param podpis: string
-- param zatwierdzono: string
UPDATE kierownik_dzialu_agenci_wpisy
SET zatwierdzil = :zatwierdzil,
  podpis = :podpis,
  zatwierdzono = :zatwierdzono
WHERE id = :id;

-- name: uspij_agenta :exec
-- Agent czeka, aż w jego sprawie coś się wydarzy.
UPDATE kierownik_dzialu_agenci_wpisy
SET stan = 'spi'
WHERE id = :id;

-- name: obudz_agenta :exec
-- Agent wraca do pracy, np. bo do jego sprawy przyszło pismo.
UPDATE kierownik_dzialu_agenci_wpisy
SET stan = 'pracuje'
WHERE id = :id;

-- name: wymien_agenta :exec
-- Wymiana agenta na następcę, którego kierownik powołał wcześniej do tej samej pracy.
-- param wymieniono: string
-- param powod_wymiany: string
-- param nastepca_id: string
UPDATE kierownik_dzialu_agenci_wpisy
SET stan = 'wymieniony',
  wymieniono = :wymieniono,
  powod_wymiany = :powod_wymiany,
  nastepca_id = :nastepca_id
WHERE id = :id;

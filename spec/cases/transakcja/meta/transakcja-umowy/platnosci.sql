-- name: platnosci_transakcji :many
-- Płatności z umów transakcji w kolejności wpisania.
SELECT *
FROM transakcja_umowy_platnosci
ORDER BY rowid;

-- name: wpisz_platnosc_transakcji :exec
-- Opłata rezerwacyjna, zadatek, zaliczka, cena albo spłata hipoteki z projektu umowy. Przy
-- kredycie także bank i warunki wypłaty. Identyfikator (UUID) nadaje aplikacja.
INSERT INTO transakcja_umowy_platnosci (id, umowa_id, rodzaj, charakter, kwota, termin, zrodlo,
  bank, warunki_wyplaty, rachunek_id)
VALUES (:id, :umowa_id, :rodzaj, :charakter, :kwota, :termin, :zrodlo, :bank, :warunki_wyplaty,
  :rachunek_id);

-- name: zmien_platnosc_transakcji :exec
-- Poprawka warunków płatności, dopóki umowa jest projektem.
UPDATE transakcja_umowy_platnosci
SET charakter = :charakter,
  kwota = :kwota,
  termin = :termin,
  zrodlo = :zrodlo,
  bank = :bank,
  warunki_wyplaty = :warunki_wyplaty
WHERE id = :id;

-- name: wskaz_rachunek_platnosci :exec
-- Rachunek, na który ma trafić wpłata; dopuszczony decyzją i niezastąpiony (R4).
-- param rachunek_id: string
UPDATE transakcja_umowy_platnosci
SET rachunek_id = :rachunek_id
WHERE id = :id;

-- name: odnotuj_wplate_platnosci :exec
-- Wpłata doszła: chwila i potwierdzenie z banku.
-- param wplacono: string
-- param potwierdzenie_dokument_id: string
UPDATE transakcja_umowy_platnosci
SET wplacono = :wplacono,
  potwierdzenie_dokument_id = :potwierdzenie_dokument_id
WHERE id = :id;

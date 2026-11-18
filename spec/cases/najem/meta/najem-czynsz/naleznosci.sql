-- name: naleznosci_najmu :many
-- Harmonogram: należności w kolejności terminu.
SELECT *
FROM najem_czynsz_naleznosci
ORDER BY termin, id;

-- name: saldo_naleznosci :many
-- Saldo każdej należności: ile pokryły wpłaty i ile zostało; z niego program liczy zaległość.
-- column zaplacono: integer
-- column zostalo: integer
SELECT
  naleznosc.id,
  naleznosc.najem_id,
  naleznosc.tytul,
  naleznosc.okres,
  naleznosc.kwota,
  naleznosc.termin,
  coalesce(sum(rozliczenie.kwota), 0) AS zaplacono,
  naleznosc.kwota - coalesce(sum(rozliczenie.kwota), 0) AS zostalo
FROM najem_czynsz_naleznosci AS naleznosc
LEFT JOIN najem_czynsz_rozliczenia AS rozliczenie ON rozliczenie.naleznosc_id = naleznosc.id
GROUP BY naleznosc.id
ORDER BY naleznosc.termin, naleznosc.id;

-- name: wpisz_naleznosc :exec
-- Należność z harmonogramu, który liczy program; identyfikator (UUID) nadaje aplikacja.
INSERT INTO najem_czynsz_naleznosci (id, najem_id, tytul, okres, kwota, termin)
VALUES (:id, :najem_id, :tytul, :okres, :kwota, :termin);

-- name: popraw_naleznosc :exec
-- Poprawka należności, dopóki nie pokrywa jej żadna wpłata.
UPDATE najem_czynsz_naleznosci
SET tytul = :tytul,
  okres = :okres,
  kwota = :kwota,
  termin = :termin
WHERE id = :id;

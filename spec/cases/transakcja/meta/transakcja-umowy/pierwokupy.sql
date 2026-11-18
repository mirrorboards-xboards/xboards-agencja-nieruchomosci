-- name: pierwokupy :many
-- Prawa pierwokupu z wynikiem; czekające mają wynik czeka.
SELECT *
FROM transakcja_umowy_pierwokupy
ORDER BY rowid;

-- name: wpisz_pierwokup :exec
-- Uprawniony z pierwokupu i podstawa prawna, przed umową przenoszącą.
INSERT INTO transakcja_umowy_pierwokupy (id, uprawniony, podstawa, zawiadomiono, wynik)
VALUES (:id, :uprawniony, :podstawa, :zawiadomiono, 'czeka');

-- name: odnotuj_zawiadomienie_o_pierwokupie :exec
-- Kancelaria zawiadomiła uprawnionego o umowie warunkowej; od tego dnia biegnie termin.
-- param zawiadomiono: string
UPDATE transakcja_umowy_pierwokupy
SET zawiadomiono = :zawiadomiono
WHERE id = :id;

-- name: odnotuj_wynik_pierwokupu :exec
-- Wynik z wiadomości kancelarii: uprawniony nie wykonał prawa, wykonał je albo termin upłynął.
-- param wynik: "nie_wykonano" | "wykonano" | "uplynal"
-- param zrodlo_wplyw_id: string
UPDATE transakcja_umowy_pierwokupy
SET wynik = :wynik,
  zrodlo_wplyw_id = :zrodlo_wplyw_id
WHERE id = :id;

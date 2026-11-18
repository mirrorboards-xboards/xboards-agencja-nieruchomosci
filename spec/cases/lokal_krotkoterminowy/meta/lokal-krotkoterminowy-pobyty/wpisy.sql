-- name: pobyty :many
-- Pobyty w kolejności przyjazdu.
SELECT *
FROM lokal_krotkoterminowy_pobyty_wpisy
ORDER BY przyjazd, przyjeto;

-- name: pobyty_w_toku :many
-- Pobyty potwierdzone, w kolizji i trwające, w kolejności przyjazdu.
SELECT *
FROM lokal_krotkoterminowy_pobyty_wpisy
WHERE stan IN ('potwierdzona', 'kolizja', 'trwa')
ORDER BY przyjazd, przyjeto;

-- name: przyjmij_rezerwacje :exec
-- Rezerwacja sprawdzona na koncie kanału: kanał, numer, termin, goście i pieniądze. Noce po końcu
-- umowy wskazują zgodę człowieka. Identyfikator (UUID) nadaje aplikacja, bo wskazują go noce,
-- zlecenia, dostępy i pozycje zestawień.
INSERT INTO lokal_krotkoterminowy_pobyty_wpisy (id, kanal, numer_rezerwacji, przyjeto, przyjazd,
  wyjazd, gosci, gosc_uczestnik_id, kwota_goscia, oplata_sprzatanie, prowizja_kanalu, oplacona,
  po_koncu_decyzja_id, stan)
VALUES (:id, :kanal, :numer_rezerwacji, :przyjeto, :przyjazd, :wyjazd, :gosci, :gosc_uczestnik_id,
  :kwota_goscia, :oplata_sprzatanie, :prowizja_kanalu, :oplacona, :po_koncu_decyzja_id,
  'potwierdzona');

-- name: zmien_rezerwacje :exec
-- Nowy termin, liczba gości albo cena po zmianie wpisanej w zmianach pobytu. Noce po końcu umowy
-- wskazują zgodę człowieka.
UPDATE lokal_krotkoterminowy_pobyty_wpisy
SET przyjazd = :przyjazd,
  wyjazd = :wyjazd,
  gosci = :gosci,
  kwota_goscia = :kwota_goscia,
  oplata_sprzatanie = :oplata_sprzatanie,
  po_koncu_decyzja_id = :po_koncu_decyzja_id
WHERE id = :id;

-- name: odnotuj_oplacenie_pobytu :exec
-- Rezerwacja opłacona według konta kanału albo rachunku biura.
UPDATE lokal_krotkoterminowy_pobyty_wpisy
SET oplacona = 1
WHERE id = :id;

-- name: odnotuj_instrukcje_wejscia :exec
-- Konsjerż wysłał instrukcję wejścia w kanale rezerwacji; tylko do opłaconej rezerwacji.
-- param instrukcja_wyslano: string
UPDATE lokal_krotkoterminowy_pobyty_wpisy
SET instrukcja_wyslano = :instrukcja_wyslano
WHERE id = :id;

-- name: zmien_stan_pobytu :exec
-- Kolizja, powrót z kolizji, początek pobytu, odwołanie.
-- param stan: "potwierdzona" | "kolizja" | "trwa" | "odwolana"
UPDATE lokal_krotkoterminowy_pobyty_wpisy
SET stan = :stan
WHERE id = :id;

-- name: zakoncz_pobyt :exec
-- Gość wyjechał; rzeczy pozostawione, jeśli są.
UPDATE lokal_krotkoterminowy_pobyty_wpisy
SET stan = 'zakonczona',
  rzeczy_pozostawione = :rzeczy_pozostawione
WHERE id = :id;

-- name: odnotuj_rozliczenie_pobytu :exec
-- Rozliczenie z konta kanału: prowizja, wypłata i jej dzień; wpisuje się raz.
-- param prowizja_kanalu: number
-- param wyplata_kanalu: number
-- param wyplacono: string
-- param rozliczono: string
UPDATE lokal_krotkoterminowy_pobyty_wpisy
SET prowizja_kanalu = :prowizja_kanalu,
  wyplata_kanalu = :wyplata_kanalu,
  wyplacono = :wyplacono,
  rozliczono = :rozliczono
WHERE id = :id;

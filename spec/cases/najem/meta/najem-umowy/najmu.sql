-- name: umowy_najmu :many
-- Umowy najmu teczki w kolejności wpisania.
SELECT *
FROM najem_umowy_najmu
ORDER BY rowid;

-- name: przygotuj_umowe_najmu :exec
-- Projekt umowy najmu według wzoru dla rodzaju najmu, który wybrał wynajmujący; identyfikator
-- (UUID) nadaje aplikacja, bo wskazują go decyzja, opłaty, aneksy i należności.
INSERT INTO najem_umowy_najmu (id, rodzaj, nieruchomosc_id, poczatek, koniec, forma,
  projekt_dokument_id, czynsz, termin_platnosci, kaucja, zwierzeta, podnajem, zmiany_w_lokalu,
  naprawy_podzial, wypowiedzenie_przypadki, stan)
VALUES (:id, :rodzaj, :nieruchomosc_id, :poczatek, :koniec, :forma, :projekt_dokument_id, :czynsz,
  :termin_platnosci, :kaucja, :zwierzeta, :podnajem, :zmiany_w_lokalu, :naprawy_podzial,
  :wypowiedzenie_przypadki, 'projekt');

-- name: zmien_projekt_umowy_najmu :exec
-- Poprawka projektu przed podpisem; po wyjściu z projektu umowę zmienia aneks.
UPDATE najem_umowy_najmu
SET poczatek = :poczatek,
  koniec = :koniec,
  forma = :forma,
  projekt_dokument_id = :projekt_dokument_id,
  czynsz = :czynsz,
  termin_platnosci = :termin_platnosci,
  kaucja = :kaucja,
  zwierzeta = :zwierzeta,
  podnajem = :podnajem,
  zmiany_w_lokalu = :zmiany_w_lokalu,
  naprawy_podzial = :naprawy_podzial,
  wypowiedzenie_przypadki = :wypowiedzenie_przypadki
WHERE id = :id;

-- name: wskaz_oswiadczenie_najemcy :exec
-- Wypis aktu z oświadczeniem najemcy (najem okazjonalny i instytucjonalny), przed podpisem.
-- param oswiadczenie_dokument_id: string
UPDATE najem_umowy_najmu
SET oswiadczenie_dokument_id = :oswiadczenie_dokument_id
WHERE id = :id;

-- name: przekaz_umowe_najmu_do_podpisu :exec
-- Projekt trafia do stron za zatwierdzoną decyzją inna, która wskazuje tę umowę.
-- param do_podpisu_decyzja_id: string
UPDATE najem_umowy_najmu
SET stan = 'do_podpisu',
  do_podpisu_decyzja_id = :do_podpisu_decyzja_id
WHERE id = :id;

-- name: odnotuj_zawarcie_umowy_najmu :exec
-- Strony podpisały umowę najmu: podpisany egzemplarz i chwila.
-- param dokument_id: string
-- param zawarto: string
UPDATE najem_umowy_najmu
SET stan = 'zawarta',
  dokument_id = :dokument_id,
  zawarto = :zawarto
WHERE id = :id;

-- name: odnotuj_przekazanie_swiadectwa :exec
-- Wynajmujący przekazał najemcy kopię świadectwa charakterystyki energetycznej.
-- param swiadectwo_przekazano: string
UPDATE najem_umowy_najmu
SET swiadectwo_przekazano = :swiadectwo_przekazano
WHERE id = :id;

-- name: odnotuj_zgloszenie_najmu_do_urzedu :exec
-- Właściciel zgłosił najem okazjonalny naczelnikowi urzędu skarbowego; potwierdzenie zgłoszenia.
-- param zgloszono_us: string
-- param zgloszenie_us_dokument_id: string
UPDATE najem_umowy_najmu
SET zgloszono_us = :zgloszono_us,
  zgloszenie_us_dokument_id = :zgloszenie_us_dokument_id
WHERE id = :id;

-- name: zakoncz_umowe_najmu :exec
-- Koniec umowy najmu albo projektu, którego strony nie podpiszą.
-- param koniec_powod: string
UPDATE najem_umowy_najmu
SET stan = 'zakonczona',
  koniec_powod = :koniec_powod
WHERE id = :id;

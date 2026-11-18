-- name: umowy :many
-- Umowy klienta z biurem w kolejności wpisania.
SELECT *
FROM sprawa_umowy_wpisy
ORDER BY rowid;

-- name: praca_od_umow :many
-- Od kiedy biuro może pracować według każdej zawartej umowy (sprawa_umowy_praca_od).
-- column od: string
SELECT
  umowa_id,
  od
FROM sprawa_umowy_praca_od
ORDER BY umowa_id;

-- name: przygotuj_umowe :exec
-- Projekt umowy pośrednictwa albo o zarządzanie; identyfikator (UUID) nadaje aplikacja, bo
-- wskazują go strony, zakres i kartoteka klientów.
INSERT INTO sprawa_umowy_wpisy (id, rodzaj, nieruchomosc_id, poczatek, koniec, okres_wypowiedzenia,
  obie_strony, wylacznosc, wynagrodzenie_opis, wynagrodzenie_zdarzenie, stan)
VALUES (:id, :rodzaj, :nieruchomosc_id, :poczatek, :koniec, :okres_wypowiedzenia, :obie_strony,
  :wylacznosc, :wynagrodzenie_opis, :wynagrodzenie_zdarzenie, 'projekt');

-- name: zmien_warunki_umowy :exec
-- Poprawka warunków projektu przed podpisem. Warunki zawartej umowy zmienia odnotuj_aneks_umowy.
UPDATE sprawa_umowy_wpisy
SET nieruchomosc_id = :nieruchomosc_id,
  poczatek = :poczatek,
  koniec = :koniec,
  okres_wypowiedzenia = :okres_wypowiedzenia,
  obie_strony = :obie_strony,
  wylacznosc = :wylacznosc,
  wynagrodzenie_opis = :wynagrodzenie_opis,
  wynagrodzenie_zdarzenie = :wynagrodzenie_zdarzenie
WHERE id = :id;

-- name: przekaz_umowe_do_podpisu :exec
-- Projekt gotowy do podpisu: strony sprawdzone, znacznik wątpliwości nie stoi.
UPDATE sprawa_umowy_wpisy
SET stan = 'do_podpisu'
WHERE id = :id;

-- name: cofnij_umowe_do_projektu :exec
-- Umowa wraca z podpisu do poprawki.
UPDATE sprawa_umowy_wpisy
SET stan = 'projekt'
WHERE id = :id;

-- name: odnotuj_zawarcie_umowy :exec
-- Umowa podpisana: forma, sposób zawarcia, podpisany dokument i kopia polisy. Przy umowie
-- z konsumentem poza lokalem biura albo na odległość także ostatni dzień terminu na odstąpienie,
-- który liczy program.
-- param forma: "pisemna" | "elektroniczna"
-- param sposob_zawarcia: "w_lokalu_biura" | "poza_lokalem" | "na_odleglosc" | "nieumowiona_wizyta"
-- param zawarto: string
-- param podpisal_za_biuro: string
-- param dokument_id: string
-- param polisa_dokument_id: string
UPDATE sprawa_umowy_wpisy
SET stan = 'zawarta',
  forma = :forma,
  sposob_zawarcia = :sposob_zawarcia,
  zawarto = :zawarto,
  podpisal_za_biuro = :podpisal_za_biuro,
  dokument_id = :dokument_id,
  polisa_dokument_id = :polisa_dokument_id,
  warunki_od = :zawarto,
  odstapienie_do = :odstapienie_do
WHERE id = :id;

-- name: odnotuj_aneks_umowy :exec
-- Nowe warunki zawartej umowy według aneksu wpisanego jako zdarzenie aneks; warunki_od to chwila
-- tego zdarzenia.
-- param warunki_od: string
UPDATE sprawa_umowy_wpisy
SET koniec = :koniec,
  okres_wypowiedzenia = :okres_wypowiedzenia,
  wylacznosc = :wylacznosc,
  wynagrodzenie_opis = :wynagrodzenie_opis,
  wynagrodzenie_zdarzenie = :wynagrodzenie_zdarzenie,
  warunki_od = :warunki_od
WHERE id = :id;

-- name: odnotuj_koniec_po_wypowiedzeniu :exec
-- Koniec zawartej umowy po zdarzeniu wypowiedzenie albo odstapienie późniejszym niż bieżące
-- warunki: wpisany albo krótszy niż dotąd, nie wcześniejszy niż dzień zdarzenia. Bez aneksu.
-- param koniec: string
UPDATE sprawa_umowy_wpisy
SET koniec = :koniec
WHERE id = :id;

-- name: odnotuj_nowa_polise_umowy :exec
-- Kopia nowej polisy pośrednika, którą wskazuje zdarzenie nowa_polisa tej umowy.
-- param polisa_dokument_id: string
UPDATE sprawa_umowy_wpisy
SET polisa_dokument_id = :polisa_dokument_id
WHERE id = :id;

-- name: odnotuj_informacje_o_odstapieniu :exec
-- Wysyłka, w której biuro poinformowało konsumenta o prawie odstąpienia; wpisuje się ją przed
-- zawarciem umowy (art. 12 upk), bo potem dane zawarcia się nie zmieniają.
-- param informacja_o_odstapieniu_wysylka_id: string
UPDATE sprawa_umowy_wpisy
SET informacja_o_odstapieniu_wysylka_id = :informacja_o_odstapieniu_wysylka_id
WHERE id = :id;

-- name: zakoncz_umowe :exec
-- Koniec umowy, np. wykonanie, wypowiedzenie, odstąpienie albo projekt bez podpisu.
-- param koniec_powod: string
UPDATE sprawa_umowy_wpisy
SET stan = 'zakonczona',
  koniec_powod = :koniec_powod
WHERE id = :id;

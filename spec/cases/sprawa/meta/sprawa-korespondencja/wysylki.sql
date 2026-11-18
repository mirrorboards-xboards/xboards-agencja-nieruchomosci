-- name: wysylki :many
-- Wysyłki teczki, najnowsze pierwsze.
SELECT *
FROM sprawa_korespondencja_wysylki
ORDER BY rowid DESC;

-- name: wysylki_do_uczestnika :many
-- Wysyłki do jednego uczestnika; z nich nowy agent wie, czy rozmówca dostał już informację RODO
-- i informację, że pisze agent.
SELECT *
FROM sprawa_korespondencja_wysylki
WHERE adresat_uczestnik_id = :adresat_uczestnik_id
ORDER BY rowid;

-- name: przygotuj_wysylke :exec
-- Projekt wysyłki z adresatem, kanałem, wzorem i plikiem treści. Wzór się potem nie zmienia;
-- inny wzór to nowy projekt. Identyfikator (UUID) nadaje aplikacja, bo wskazują go decyzja
-- i załączniki.
INSERT INTO sprawa_korespondencja_wysylki (id, adresat_uczestnik_id, adresat_rodzaj, adresat_nazwa,
  w_imieniu, w_imieniu_uczestnik_id, umocowanie_id, czynnosc, kanal, wzor, wzor_wersja,
  zamawia_dokument_id, sciezka, skrot, informacja_rodo, ujawnia_agenta, stan)
VALUES (:id, :adresat_uczestnik_id, :adresat_rodzaj, :adresat_nazwa, :w_imieniu,
  :w_imieniu_uczestnik_id, :umocowanie_id, :czynnosc, :kanal, :wzor, :wzor_wersja,
  :zamawia_dokument_id, :sciezka, :skrot, :informacja_rodo, :ujawnia_agenta, 'projekt');

-- name: zmien_tresc_wysylki :exec
-- Nowa wersja pliku treści; zmienia się tylko w projekcie, przed prośbą o zgodę i wysłaniem.
UPDATE sprawa_korespondencja_wysylki
SET sciezka = :sciezka,
  skrot = :skrot
WHERE id = :id;

-- name: przekaz_wysylke_do_zgody :exec
-- Wysyłka, która według profilu wymaga zgody człowieka, i każde pismo spoza wzorów czeka na
-- decyzję „wyslanie”. Od tej chwili treść się nie zmienia, a o decyzję prosi się dopiero teraz.
UPDATE sprawa_korespondencja_wysylki
SET stan = 'czeka_na_zgode'
WHERE id = :id;

-- name: odnotuj_wyslanie :exec
-- Wysyłka wyszła; pismo za zgodą wskazuje zatwierdzoną decyzję.
-- param wyslano: string
UPDATE sprawa_korespondencja_wysylki
SET stan = 'wyslana',
  wyslano = :wyslano,
  decyzja_id = :decyzja_id,
  odnosnik_zewnetrzny = :odnosnik_zewnetrzny
WHERE id = :id;

-- name: odnotuj_doreczenie :exec
-- Adresat dostał wysyłkę; dowód, gdy kanał go daje.
-- param doreczono: string
UPDATE sprawa_korespondencja_wysylki
SET stan = 'doreczona',
  doreczono = :doreczono,
  dowod_doreczenia = :dowod_doreczenia
WHERE id = :id;

-- name: odnotuj_zwrot :exec
-- Wysyłka wróciła niedoręczona.
UPDATE sprawa_korespondencja_wysylki
SET stan = 'zwrocona',
  dowod_doreczenia = :dowod_doreczenia
WHERE id = :id;

-- name: anuluj_wysylke :exec
-- Projekt, który nie wyjdzie.
UPDATE sprawa_korespondencja_wysylki
SET stan = 'anulowana'
WHERE id = :id;

-- name: oferty :many
-- Oferty zainteresowanych i kontroferty klienta w kolejności wpisania.
SELECT *
FROM oferta_oferty_wpisy
ORDER BY wpisano, rowid;

-- name: oferty_do_przedstawienia :many
-- Oferty zainteresowanych, których Opiekun jeszcze nie przedstawił klientowi.
SELECT *
FROM oferta_oferty_wpisy
WHERE kierunek = 'od_zainteresowanego'
  AND przedstawiono IS NULL
ORDER BY wpisano, rowid;

-- name: wpisz_oferte_zainteresowanego :exec
-- Oferta zainteresowanego z wpływem albo notatką pośrednika, z warunkami i terminem związania
-- albo sposobem złożenia. Nowa oferta w łańcuchu wskazuje kontrofertę klienta, na którą
-- odpowiada. Identyfikator (UUID) nadaje aplikacja, bo wskazują go zgoda, decyzja i katalog ofert.
INSERT INTO oferta_oferty_wpisy (id, dotyczy, kierunek, osoba_id, odpowiada_na_id, wplyw_id,
  dokument_id, wpisano, kwota, zadatek_albo_kaucja, termin_umowy, termin_wydania, finansowanie,
  warunki_zastrzezone, wiaze_do, sposob_zlozenia)
VALUES (:id, :dotyczy, 'od_zainteresowanego', :osoba_id, :odpowiada_na_id, :wplyw_id, :dokument_id,
  :wpisano, :kwota, :zadatek_albo_kaucja, :termin_umowy, :termin_wydania, :finansowanie,
  :warunki_zastrzezone, :wiaze_do, :sposob_zlozenia);

-- name: odnotuj_przedstawienie_oferty :exec
-- Chwila, w której klient dostał ofertę; oferta klienta biura bez zgody umowy na obie strony
-- wskazuje zatwierdzoną decyzję człowieka.
-- param przedstawiono: string
UPDATE oferta_oferty_wpisy
SET przedstawiono = :przedstawiono,
  obie_strony_decyzja_id = :obie_strony_decyzja_id
WHERE id = :id;

-- name: odnotuj_decyzje_klienta_o_ofercie :exec
-- Przyjęcie, odrzucenie albo kontroferta klienta z jego zgodą odpowiedz_na_oferte.
-- param decyzja_klienta: "przyjeta" | "odrzucona" | "kontroferta"
-- param zgoda_id: string
UPDATE oferta_oferty_wpisy
SET decyzja_klienta = :decyzja_klienta,
  zgoda_id = :zgoda_id
WHERE id = :id;

-- name: wpisz_kontroferte_klienta :exec
-- Kontroferta klienta jako nowy wpis, który wskazuje ofertę zainteresowanego i zgodę z decyzji
-- klienta.
-- param odpowiada_na_id: string
-- param zgoda_id: string
INSERT INTO oferta_oferty_wpisy (id, dotyczy, kierunek, osoba_id, odpowiada_na_id, wpisano, kwota,
  zadatek_albo_kaucja, termin_umowy, termin_wydania, warunki_zastrzezone, wiaze_do,
  sposob_zlozenia, zgoda_id)
VALUES (:id, :dotyczy, 'od_klienta', :osoba_id, :odpowiada_na_id, :wpisano, :kwota,
  :zadatek_albo_kaucja, :termin_umowy, :termin_wydania, :warunki_zastrzezone, :wiaze_do,
  :sposob_zlozenia, :zgoda_id);

-- name: wskaz_odpowiedz_na_oferte :exec
-- Wysyłka z odpowiedzią na ofertę zainteresowanego; jeszcze niewysłana, bo wychodzi tylko za
-- zgodą człowieka.
-- param odpowiedz_wysylka_id: string
UPDATE oferta_oferty_wpisy
SET odpowiedz_wysylka_id = :odpowiedz_wysylka_id
WHERE id = :id;

-- name: odnotuj_odpowiedz_zainteresowanego :exec
-- Odpowiedź zainteresowanego na kontrofertę klienta, z wpływem, w którym przyszła.
-- param odpowiedz_drugiej_strony: "przyjeta" | "odrzucona" | "kontroferta"
-- param odpowiedz_wplyw_id: string
UPDATE oferta_oferty_wpisy
SET odpowiedz_drugiej_strony = :odpowiedz_drugiej_strony,
  odpowiedz_wplyw_id = :odpowiedz_wplyw_id
WHERE id = :id;

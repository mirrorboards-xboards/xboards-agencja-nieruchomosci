-- name: oferty_klienta :many
-- Oferty klienta i kontroferty drugiej strony w kolejności wpisania.
SELECT *
FROM poszukiwanie_oferty_wpisy
ORDER BY wpisano, rowid;

-- name: przygotuj_oferte_klienta :exec
-- Oferta albo kontroferta klienta według wzoru biura, przed jego potwierdzeniem; kontroferta
-- wskazuje kontrofertę drugiej strony z decyzją klienta „kontroferta”. Identyfikator (UUID)
-- nadaje aplikacja, bo wskazuje go zgoda klienta.
INSERT INTO poszukiwanie_oferty_wpisy (id, dotyczy, kierunek, propozycja_id, adresat,
  odpowiada_na_id, wpisano, kwota, zadatek_albo_kaucja, termin_umowy, termin_wydania,
  finansowanie_w_ofercie, warunki_zastrzezone, wiaze_do, sposob_zlozenia)
VALUES (:id, :dotyczy, 'od_klienta', :propozycja_id, :adresat, :odpowiada_na_id, :wpisano, :kwota,
  :zadatek_albo_kaucja, :termin_umowy, :termin_wydania, :finansowanie_w_ofercie,
  :warunki_zastrzezone, :wiaze_do, :sposob_zlozenia);

-- name: potwierdz_oferte_klienta :exec
-- Klient potwierdził treść swojej oferty zgodą potwierdzenie_oferty.
-- param zgoda_id: string
UPDATE poszukiwanie_oferty_wpisy
SET zgoda_id = :zgoda_id
WHERE id = :id;

-- name: wskaz_wysylke_oferty_klienta :exec
-- Wysyłka potwierdzonej oferty klienta; jeszcze niewysłana, bo wychodzi tylko za zgodą
-- człowieka.
-- param wysylka_id: string
UPDATE poszukiwanie_oferty_wpisy
SET wysylka_id = :wysylka_id
WHERE id = :id;

-- name: odnotuj_odpowiedz_drugiej_strony :exec
-- Przyjęcie, odrzucenie albo kontroferta drugiej strony z wpływem i chwilą, w której klient je
-- poznał.
-- param odpowiedz_drugiej_strony: "przyjeta" | "odrzucona" | "kontroferta"
-- param odpowiedz_wplyw_id: string
UPDATE poszukiwanie_oferty_wpisy
SET odpowiedz_drugiej_strony = :odpowiedz_drugiej_strony,
  odpowiedz_wplyw_id = :odpowiedz_wplyw_id,
  odpowiedz_przedstawiono = :odpowiedz_przedstawiono
WHERE id = :id;

-- name: wpisz_kontroferte_drugiej_strony :exec
-- Kontroferta drugiej strony z wpływem, która odpowiada na ofertę klienta.
-- param odpowiada_na_id: string
-- param wplyw_id: string
INSERT INTO poszukiwanie_oferty_wpisy (id, dotyczy, kierunek, propozycja_id, adresat,
  odpowiada_na_id, wplyw_id, wpisano, kwota, zadatek_albo_kaucja, termin_umowy, termin_wydania,
  finansowanie_w_ofercie, warunki_zastrzezone, wiaze_do, sposob_zlozenia)
VALUES (:id, :dotyczy, 'od_drugiej_strony', :propozycja_id, :adresat, :odpowiada_na_id, :wplyw_id,
  :wpisano, :kwota, :zadatek_albo_kaucja, :termin_umowy, :termin_wydania, :finansowanie_w_ofercie,
  :warunki_zastrzezone, :wiaze_do, :sposob_zlozenia);

-- name: odnotuj_przedstawienie_kontroferty :exec
-- Chwila, w której klient dostał kontrofertę drugiej strony.
-- param przedstawiono: string
UPDATE poszukiwanie_oferty_wpisy
SET przedstawiono = :przedstawiono
WHERE id = :id;

-- name: odnotuj_decyzje_klienta_o_kontrofercie :exec
-- Przyjęcie albo odrzucenie kontroferty ze zgodą potwierdzenie_oferty albo kontroferta klienta,
-- którą klient potwierdza na jej własnym wpisie.
-- param decyzja_klienta: "przyjeta" | "odrzucona" | "kontroferta"
UPDATE poszukiwanie_oferty_wpisy
SET decyzja_klienta = :decyzja_klienta,
  zgoda_id = :zgoda_id
WHERE id = :id;

-- name: wskaz_odpowiedz_na_kontroferte :exec
-- Wysyłka z przyjęciem albo odrzuceniem kontroferty; jeszcze niewysłana, bo wychodzi tylko za
-- zgodą człowieka.
-- param odpowiedz_wysylka_id: string
UPDATE poszukiwanie_oferty_wpisy
SET odpowiedz_wysylka_id = :odpowiedz_wysylka_id
WHERE id = :id;

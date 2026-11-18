-- name: umowy_klienta :many
-- Umowy klienta z biurem z bieżącym stanem, najnowsze pierwsze. Kierownik pośrednictwa sprawdza
-- tu obie_strony przed transakcją dla obu stron (R1).
SELECT
  umowy.*,
  stany.stan,
  stany.od AS stan_od
FROM klienci_kartoteka_umowy AS umowy
JOIN klienci_kartoteka_umowy_stany AS stany ON stany.umowa_id = umowy.id
  AND stany.id = (
    SELECT max(id)
    FROM klienci_kartoteka_umowy_stany
    WHERE umowa_id = umowy.id
  )
WHERE umowy.klient_id = :klient_id
ORDER BY umowy.zawarto DESC, umowy.rowid DESC;

-- name: stany_umowy_klienta :many
-- Historia stanów jednej umowy ze źródłami.
SELECT *
FROM klienci_kartoteka_umowy_stany
WHERE umowa_id = :umowa_id
ORDER BY id;

-- name: koniec_ostatniej_umowy_klienta :one
-- Dzień końca ostatniej umowy klienta; null, gdy któraś umowa jeszcze obowiązuje albo klient nie
-- ma umów.
-- column koniec: string | null
SELECT
  CASE
    WHEN count(*) = 0 OR sum(stany.stan = 'obowiazuje') > 0 THEN NULL
    ELSE max(stany.od)
  END AS koniec
FROM klienci_kartoteka_umowy AS umowy
JOIN klienci_kartoteka_umowy_stany AS stany ON stany.umowa_id = umowy.id
  AND stany.id = (
    SELECT max(id)
    FROM klienci_kartoteka_umowy_stany
    WHERE umowa_id = umowy.id
  )
WHERE umowy.klient_id = :klient_id;

-- name: wpisz_umowe_klienta :exec
-- Umowa klienta z biurem w dniu jej podpisania; obowiązuje od zawarcia, a źródłem stanu jest
-- umowa w teczce. Identyfikator (UUID) nadaje aplikacja.
INSERT INTO klienci_kartoteka_umowy (id, klient_id, rodzaj, dzial, nieruchomosc_adres, numer_kw,
  teczka_odnosnik, typ_teczki, oznaczenie, umowa_odnosnik, podpisal_za_biuro, zawarto, koniec,
  konsument, obie_strony, wpisal, wpisano)
VALUES (:id, :klient_id, :rodzaj, :dzial, :nieruchomosc_adres, :numer_kw, :teczka_odnosnik,
  :typ_teczki, :oznaczenie, :umowa_odnosnik, :podpisal_za_biuro, :zawarto, :koniec, :konsument,
  :obie_strony, :wpisal, :wpisano);
INSERT INTO klienci_kartoteka_umowy_stany (umowa_id, stan, od, zrodlo_odnosnik, wpisal)
VALUES (:id, 'obowiazuje', :zawarto, :umowa_odnosnik, :wpisal);

-- name: zmien_stan_umowy_klienta :exec
-- Koniec umowy po raporcie agenta: odstąpienie, wypowiedzenie, wygaśnięcie albo wykonanie, ze
-- źródłem w teczce.
INSERT INTO klienci_kartoteka_umowy_stany (umowa_id, stan, od, zrodlo_odnosnik, wpisal)
VALUES (:umowa_id, :stan, :od, :zrodlo_odnosnik, :wpisal);

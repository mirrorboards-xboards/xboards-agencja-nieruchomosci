-- name: karty_czekajace :many
-- Karty, które czekają na Kierownika biura, od najstarszej.
SELECT *
FROM recepcja_karty_wpisy
WHERE stan = 'czeka'
ORDER BY wyslano, id;

-- name: wyslij_karte :exec
-- Karta wysłana Kierownikowi biura z wpisu dziennika oddanego na kartę.
INSERT INTO recepcja_karty_wpisy (rodzaj, wplyw_id, wyslano, rodzaj_sprawy, rodzaj_nieruchomosci,
  miejscowosc, kanal, zglaszajacy, kontakt, wiadomosc_odnosnik, stan)
VALUES (:rodzaj, :wplyw_id, :wyslano, :rodzaj_sprawy, :rodzaj_nieruchomosci, :miejscowosc, :kanal,
  :zglaszajacy, :kontakt, :wiadomosc_odnosnik, 'czeka');

-- name: odnotuj_rozstrzygniecie_karty :exec
-- Kierownik biura przydzielił kartę działowi albo ją odrzucił; dane zgłaszającego znikają z karty.
-- param rozstrzygnieto: string
-- param rozstrzygniecie: "przydzielona" | "odrzucona"
UPDATE recepcja_karty_wpisy
SET stan = 'rozstrzygnieta',
  rozstrzygnieto = :rozstrzygnieto,
  rozstrzygniecie = :rozstrzygniecie,
  dzial = :dzial,
  zglaszajacy = NULL,
  kontakt = NULL
WHERE id = :id;

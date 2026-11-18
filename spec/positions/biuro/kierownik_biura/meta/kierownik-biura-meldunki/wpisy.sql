-- name: ostatnie_meldunki :many
-- Ostatni meldunek każdego nadawcy; z niego kierownik zna bieżące obłożenie.
SELECT meldunek.*
FROM kierownik_biura_meldunki_wpisy AS meldunek
WHERE meldunek.id = (
  SELECT ostatni.id
  FROM kierownik_biura_meldunki_wpisy AS ostatni
  WHERE ostatni.nadawca = meldunek.nadawca
  ORDER BY ostatni.otrzymano DESC, ostatni.id DESC
  LIMIT 1
)
ORDER BY meldunek.nadawca;

-- name: meldunki_od :many
-- Meldunki od jednego dnia, od najstarszego; z nich powstaje przebieg obłożenia w sprawozdaniu.
-- param od: string
SELECT *
FROM kierownik_biura_meldunki_wpisy
WHERE otrzymano >= :od
ORDER BY otrzymano, id;

-- name: dopisz_meldunek_dzialu :exec
-- Meldunek kierownika Pośrednictwa, Transakcji albo Najmu.
-- param nadawca: "posrednictwo" | "transakcje" | "najem"
INSERT INTO kierownik_biura_meldunki_wpisy (nadawca, tozsamosc, otrzymano, agenci_pracuja,
  sprawy_zablokowane, decyzje_czekaja, najblizszy_termin, twierdzenia)
VALUES (:nadawca, :tozsamosc, :otrzymano, :agenci_pracuja, :sprawy_zablokowane, :decyzje_czekaja,
  :najblizszy_termin, :twierdzenia);

-- name: dopisz_meldunek_recepcji :exec
-- Dzienny meldunek Recepcji.
INSERT INTO kierownik_biura_meldunki_wpisy (nadawca, tozsamosc, otrzymano, wiadomosci_wplynely,
  karty_wyslane, zaklocenia, twierdzenia)
VALUES ('recepcja', :tozsamosc, :otrzymano, :wiadomosci_wplynely, :karty_wyslane, :zaklocenia,
  :twierdzenia);

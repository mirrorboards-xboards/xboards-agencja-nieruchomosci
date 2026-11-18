-- name: meldunek_dzienny :one
-- Stałe pola meldunku Recepcji z jednego dnia (RRRR-MM-DD): ile wiadomości wpłynęło, ile oddano
-- agentom spraw, ile zapytań z portali i ze strony biura czekało dłużej niż godzinę, ile kart
-- wysłano, ile kart czeka od wcześniejszego dnia i ile wiadomości było niejednoznacznych. Kanały
-- podaje wplywy_wedlug_kanalow, a zakłócenia zdarzenia_zrodel. Meldunek jest wiadomością; pamięć
-- trzyma tylko to, z czego się go liczy.
-- param dzien: string
-- column wplynelo: integer
-- column oddano_agentom: integer
-- column zapytania_po_godzinie: integer
-- column wyslano_kart: integer
-- column karty_czekaja_od_wczesniej: integer
-- column niejednoznaczne: integer
SELECT
  (
    SELECT count(*)
    FROM recepcja_wplywy_dziennik
    WHERE substr(wplynelo, 1, 10) = :dzien
  ) AS wplynelo,
  (
    SELECT count(*)
    FROM recepcja_wplywy_dziennik
    WHERE substr(wplynelo, 1, 10) = :dzien
      AND oddano = 'agent_sprawy'
  ) AS oddano_agentom,
  (
    SELECT count(*)
    FROM recepcja_wplywy_dziennik
    WHERE substr(wplynelo, 1, 10) = :dzien
      AND kanal IN ('portal', 'formularz')
      AND oddano_o IS NOT NULL
      AND julianday(oddano_o) - julianday(wplynelo) > 1.0 / 24
  ) AS zapytania_po_godzinie,
  (
    SELECT count(*)
    FROM recepcja_karty_wpisy
    WHERE substr(wyslano, 1, 10) = :dzien
  ) AS wyslano_kart,
  (
    SELECT count(*)
    FROM recepcja_karty_wpisy
    WHERE stan = 'czeka'
      AND substr(wyslano, 1, 10) < :dzien
  ) AS karty_czekaja_od_wczesniej,
  (
    SELECT count(*)
    FROM recepcja_karty_wpisy
    WHERE substr(wyslano, 1, 10) = :dzien
      AND rodzaj = 'pismo_bez_sprawy'
  ) AS niejednoznaczne;

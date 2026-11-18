-- Reguły teczki oferty na tabelach rdzenia: typ pokoju, etapy z przejściami, role, rodzaje zgód,
-- rodzaj umowy z biurem i zakaz pieniędzy. Rodzaj nie ma własnych tabel; wskazuje tylko rdzeń,
-- więc jest pierwszym rodzajem typu.
--
-- Reguły na rdzeniu, które zależą od tabel typu, leżą w migracjach tych rodzajów. Decyzje własne
-- typu (pierwsza publikacja, publikacja mimo rozbieżności z księgą, oferta klienta biura bez zgody
-- na obie strony) są decyzjami „inna”, które wskazują wiersz typu; ich reguły leżą w rodzajach
-- oferta-ogloszenia i oferta-oferty.
--
-- Przerwana sprawa się zamyka albo wraca do etapu sprzed przerwania za decyzją wznowienie_sprawy
-- (rdzeń, sprawa_metryka_po_przerwaniu_zamkniecie_albo_powrot, T1).

-- Teczka oferty ma typ pokoju oferta; metryka innego typu w tym pokoju pomieszałaby reguły typów.
CREATE TRIGGER oferta_reguly_typ_pokoju
BEFORE INSERT ON sprawa_metryka_wpis
WHEN NEW.typ_pokoju <> 'oferta'
BEGIN
  SELECT RAISE(ABORT, 'teczka oferty ma typ pokoju oferta');
END;

-- Etapy oferty z rozdziału „Etapy” profilu: przyjeta, weryfikacja_klienta, umowa_posrednictwa,
-- przygotowanie_oferty, oferta_aktywna i oferta_przyjeta, która może wrócić do oferta_aktywna,
-- gdy przyjęta oferta nie doszła do skutku. Od weryfikacji do przyjętej oferty sprawa przechodzi
-- do koniec_oferty, a z niego do zamknieta. Do przerwana przechodzi się z każdego etapu przed
-- zamknięciem, a z przerwana do zamknieta albo z powrotem do etapu sprzed przerwania
-- (sprawa_metryka_powrot_z_przerwania). Pierwszy etap (przyjeta) i decyzje przerwania i wznowienia
-- pilnuje rdzeń.
CREATE TRIGGER oferta_reguly_przejscia_etapow
BEFORE INSERT ON sprawa_metryka_etapy
WHEN EXISTS (
    SELECT 1
    FROM sprawa_metryka_etapy
  )
  AND NOT coalesce(
    (
      SELECT
        (poprzedni.etap = 'przyjeta' AND NEW.etap = 'weryfikacja_klienta')
        OR (poprzedni.etap = 'weryfikacja_klienta' AND NEW.etap = 'umowa_posrednictwa')
        OR (poprzedni.etap = 'umowa_posrednictwa' AND NEW.etap = 'przygotowanie_oferty')
        OR (poprzedni.etap = 'przygotowanie_oferty' AND NEW.etap = 'oferta_aktywna')
        OR (poprzedni.etap = 'oferta_aktywna' AND NEW.etap = 'oferta_przyjeta')
        OR (poprzedni.etap = 'oferta_przyjeta' AND NEW.etap = 'oferta_aktywna')
        OR (
          poprzedni.etap IN ('weryfikacja_klienta', 'umowa_posrednictwa', 'przygotowanie_oferty', 'oferta_aktywna', 'oferta_przyjeta')
          AND NEW.etap = 'koniec_oferty'
        )
        OR (poprzedni.etap = 'koniec_oferty' AND NEW.etap = 'zamknieta')
        OR (poprzedni.etap NOT IN ('przerwana', 'zamknieta') AND NEW.etap = 'przerwana')
        OR (
          poprzedni.etap = 'przerwana'
          AND (
            NEW.etap = 'zamknieta'
            OR NEW.etap IN (
              SELECT etap
              FROM sprawa_metryka_powrot_z_przerwania
            )
          )
        )
      FROM sprawa_metryka_etapy AS poprzedni
      ORDER BY poprzedni.id DESC
      LIMIT 1
    ),
    0
  )
BEGIN
  SELECT RAISE(ABORT, 'etap oferty przechodzi: przyjeta, weryfikacja_klienta, umowa_posrednictwa, przygotowanie_oferty, oferta_aktywna i oferta_przyjeta, potem koniec_oferty i zamknieta; przerwana z każdego etapu, a po niej zamknieta albo powrót do etapu sprzed przerwania');
END;

-- Pracę nad ofertą i jej ogłaszanie zaczyna podpisana umowa pośrednictwa (art. 180 ust. 3 ugn):
-- bez niej nie ma ani przygotowania oferty, ani oferty aktywnej. Powrót z przyjętej oferty do
-- aktywnej też wymaga, żeby umowa jeszcze trwała.
CREATE TRIGGER oferta_reguly_etap_oferty_po_zawarciu_umowy
BEFORE INSERT ON sprawa_metryka_etapy
WHEN NEW.etap IN ('przygotowanie_oferty', 'oferta_aktywna')
  AND NOT EXISTS (
    SELECT 1
    FROM sprawa_umowy_wpisy
    WHERE rodzaj = 'posrednictwo'
      AND stan = 'zawarta'
  )
BEGIN
  SELECT RAISE(ABORT, 'etapy przygotowanie_oferty i oferta_aktywna wymagają zawartej umowy pośrednictwa');
END;

-- Role w teczce oferty: klient (strona umowy), współwłaściciel z księgi, który nie jest stroną
-- umowy, i małżonek. Zainteresowani nie są uczestnikami rdzenia, tylko osobami
-- w oferta-zainteresowani; pełnomocnik wynika z umocowań.
CREATE TRIGGER oferta_reguly_role
BEFORE INSERT ON sprawa_uczestnicy_role
WHEN NEW.rola NOT IN ('klient', 'wspolwlasciciel', 'malzonek')
BEGIN
  SELECT RAISE(ABORT, 'rola w teczce oferty to klient, wspolwlasciciel albo malzonek');
END;

-- Zgody klienta z rozdziału „Sprawy za zgodą i zakazane” i zgody rdzenia.
CREATE TRIGGER oferta_reguly_rodzaje_zgod
BEFORE INSERT ON sprawa_zgody_wpisy
WHEN NEW.rodzaj NOT IN (
  'zadanie_rozpoczecia_pracy',
  'odstapienie',
  'zatwierdzenie_ceny',
  'granice_negocjacji',
  'zatwierdzenie_opisu',
  'warunki_najmu',
  'odpowiedz_na_oferte',
  'ujawnienie_informacji'
)
BEGIN
  SELECT RAISE(ABORT, 'zgoda w teczce oferty to zatwierdzenie_ceny, granice_negocjacji, zatwierdzenie_opisu, warunki_najmu, odpowiedz_na_oferte, ujawnienie_informacji albo zgoda rdzenia');
END;

-- Teczka oferty ma umowę pośrednictwa; umowę o zarządzanie prowadzi teczka najmu.
CREATE TRIGGER oferta_reguly_umowa_posrednictwa
BEFORE INSERT ON sprawa_umowy_wpisy
WHEN NEW.rodzaj <> 'posrednictwo'
BEGIN
  SELECT RAISE(ABORT, 'umowa w teczce oferty to umowa pośrednictwa');
END;

-- Teczka oferty nie przyjmuje ani nie wypłaca pieniędzy: opłata rezerwacyjna, zadatek i kaucja
-- należą do teczki transakcji albo najmu, gdzie rachunek sprawdza Kontroler (R4).
CREATE TRIGGER oferta_reguly_bez_rachunkow
BEFORE INSERT ON sprawa_rachunki_wpisy
BEGIN
  SELECT RAISE(ABORT, 'teczka oferty nie przyjmuje pieniędzy: nie ma rachunków ani wypłat');
END;

CREATE TRIGGER oferta_reguly_bez_wyplat
BEFORE INSERT ON sprawa_wyplaty_wpisy
BEGIN
  SELECT RAISE(ABORT, 'teczka oferty nie przyjmuje pieniędzy: nie ma rachunków ani wypłat');
END;

//! The kind `najem-reguly` of the room `cases/najem`: its migrations, the values of its enumerated
//! columns and its operations.

/// The kind's name.
pub const KIND: &str = "najem-reguly";

/// The migrations of the kind, in the order of their numbers.
pub const MIGRATIONS: &[&str] = &[
    r#"-- Reguły teczki najmu na tabelach rdzenia: typ pokoju, etapy z przejściami, role uczestników,
-- rodzaje zgód i rodzaj umowy z biurem. Rodzaj nie ma własnych tabel i wskazuje tylko rdzeń, więc
-- jest pierwszym rodzajem typu. Każdy trigger tylko odmawia; rdzenia nie zmienia.
--
-- Etapy według profilu:
--   przyjeta → [umowa_z_biurem →] projekt_umowy → [oswiadczenie_najemcy →] podpisanie_umowy
--   → wydanie_lokalu → [zgloszenie_najmu →] trwanie_najmu → koniec_najmu → zwrot_lokalu
--   → [rozliczenie_kaucji →] po_najmie → zamknieta.
-- Przy samym pośrednictwie (bez umowy o zarządzanie, R6) sprawa kończy się po wydaniu lokalu
-- (albo po zgłoszeniu najmu okazjonalnego): przechodzi do po_najmie albo zamknieta. Z każdego etapu
-- przed zamknięciem sprawa przechodzi do przerwana, a z niej do zamknieta albo z powrotem do etapu
-- sprzed przerwania (sprawa_metryka_powrot_z_przerwania). Decyzje przerwanie_sprawy
-- i wznowienie_sprawy wymaga rdzeń.
--
-- Reguły etapów, które zależą od tabel typu, leżą w ich migracjach:
-- - oświadczenie najemcy pomija się tylko przy najmie zwykłym, a zgłoszenie do urzędu tylko przy
--   innym niż okazjonalny: najem-umowy (najem_umowy_pominiecie_oswiadczenia_przy_zwyklym,
--   najem_umowy_okazjonalny_przez_zgloszenie);
-- - rozliczenie kaucji pomija się tylko bez pobranej kaucji: najem-kaucja
--   (najem_kaucja_rozliczenie_przy_pobranej).

-- Teczka najmu to pokój typu najem (decyzja T2): reguły tego katalogu dotyczą tylko jej.
CREATE TRIGGER najem_reguly_typ_pokoju
BEFORE INSERT ON sprawa_metryka_wpis
WHEN NEW.typ_pokoju <> 'najem'
BEGIN
  SELECT RAISE(ABORT, 'teczka najmu ma typ pokoju najem');
END;

-- Etap najmu i przejście z bieżącego etapu (ostatniego wiersza dziennika): etap spoza listy nie ma
-- dozwolonego przejścia. Pierwszy wpis (przyjeta, którego pilnuje rdzeń) nie ma poprzedniego
-- etapu, więc coalesce go przepuszcza.
CREATE TRIGGER najem_reguly_przejscie_etapu
BEFORE INSERT ON sprawa_metryka_etapy
WHEN NOT coalesce(
  (
    SELECT
      CASE etap
        WHEN 'przyjeta' THEN NEW.etap IN ('umowa_z_biurem', 'projekt_umowy', 'przerwana')
        WHEN 'umowa_z_biurem' THEN NEW.etap IN ('projekt_umowy', 'przerwana')
        WHEN 'projekt_umowy' THEN NEW.etap IN ('oswiadczenie_najemcy', 'podpisanie_umowy', 'przerwana')
        WHEN 'oswiadczenie_najemcy' THEN NEW.etap IN ('podpisanie_umowy', 'przerwana')
        WHEN 'podpisanie_umowy' THEN NEW.etap IN ('wydanie_lokalu', 'przerwana')
        WHEN 'wydanie_lokalu' THEN NEW.etap IN ('zgloszenie_najmu', 'trwanie_najmu', 'po_najmie', 'zamknieta', 'przerwana')
        WHEN 'zgloszenie_najmu' THEN NEW.etap IN ('trwanie_najmu', 'po_najmie', 'zamknieta', 'przerwana')
        WHEN 'trwanie_najmu' THEN NEW.etap IN ('koniec_najmu', 'przerwana')
        WHEN 'koniec_najmu' THEN NEW.etap IN ('zwrot_lokalu', 'przerwana')
        WHEN 'zwrot_lokalu' THEN NEW.etap IN ('rozliczenie_kaucji', 'po_najmie', 'przerwana')
        WHEN 'rozliczenie_kaucji' THEN NEW.etap IN ('po_najmie', 'przerwana')
        WHEN 'po_najmie' THEN NEW.etap IN ('zamknieta', 'przerwana')
        WHEN 'przerwana' THEN NEW.etap = 'zamknieta' OR NEW.etap IN (
          SELECT etap
          FROM sprawa_metryka_powrot_z_przerwania
        )
        ELSE 0
      END
    FROM sprawa_metryka_etapy
    ORDER BY id DESC
    LIMIT 1
  ),
  1
)
BEGIN
  SELECT RAISE(ABORT, 'etapy najmu idą po kolei: przyjeta, umowa_z_biurem (przy zarządzaniu), projekt_umowy, oswiadczenie_najemcy (przy okazjonalnym i instytucjonalnym), podpisanie_umowy, wydanie_lokalu, zgloszenie_najmu (przy okazjonalnym), trwanie_najmu, koniec_najmu, zwrot_lokalu, rozliczenie_kaucji (przy kaucji), po_najmie, zamknieta; każdy etap przed zamknięciem można przerwać (przerwana), a po przerwaniu przychodzi zamknieta albo powrót do etapu sprzed przerwania');
END;

-- R6: przy umowie o zarządzanie biuro prowadzi najem do rozliczenia kaucji, więc po wydaniu lokalu
-- sprawa nie kończy się od razu. Kończy się tak tylko samo pośrednictwo.
CREATE TRIGGER najem_reguly_koniec_po_wydaniu_tylko_bez_zarzadzania
BEFORE INSERT ON sprawa_metryka_etapy
WHEN NEW.etap IN ('po_najmie', 'zamknieta')
  AND (
    SELECT etap IN ('wydanie_lokalu', 'zgloszenie_najmu')
    FROM sprawa_metryka_etapy
    ORDER BY id DESC
    LIMIT 1
  )
  AND EXISTS (
    SELECT 1
    FROM sprawa_umowy_wpisy
    WHERE rodzaj = 'zarzadzanie'
      AND stan = 'zawarta'
  )
BEGIN
  SELECT RAISE(ABORT, 'przy zawartej umowie o zarządzanie najem nie kończy się po wydaniu lokalu (R6)');
END;

-- Art. 675 § 3 KC: bez protokołu domniemywa się, że lokal wydano w dobrym stanie, więc sprawa
-- wychodzi z wydania lokalu tylko z podpisanym protokołem wydania. To samo przy zwrocie: protokół
-- zwrotu porównuje się z protokołem wydania. Przerwać sprawę wolno bez protokołu.
CREATE TRIGGER najem_reguly_wyjscie_z_wydania_z_protokolem
BEFORE INSERT ON sprawa_metryka_etapy
WHEN NEW.etap <> 'przerwana'
  AND (
    SELECT etap = 'wydanie_lokalu'
    FROM sprawa_metryka_etapy
    ORDER BY id DESC
    LIMIT 1
  )
  AND NOT EXISTS (
    SELECT 1
    FROM sprawa_protokoly_wpisy
    WHERE rodzaj = 'wydanie'
      AND stan = 'podpisany'
  )
BEGIN
  SELECT RAISE(ABORT, 'sprawa wychodzi z wydania lokalu tylko z podpisanym protokołem wydania');
END;

CREATE TRIGGER najem_reguly_wyjscie_ze_zwrotu_z_protokolem
BEFORE INSERT ON sprawa_metryka_etapy
WHEN NEW.etap <> 'przerwana'
  AND (
    SELECT etap = 'zwrot_lokalu'
    FROM sprawa_metryka_etapy
    ORDER BY id DESC
    LIMIT 1
  )
  AND NOT EXISTS (
    SELECT 1
    FROM sprawa_protokoly_wpisy
    WHERE rodzaj = 'zwrot'
      AND stan = 'podpisany'
  )
BEGIN
  SELECT RAISE(ABORT, 'sprawa wychodzi ze zwrotu lokalu tylko z podpisanym protokołem zwrotu');
END;

-- Pełnomocnik wynika z umocowań, a klient biura ze znacznika uczestnika. Właściciel innego lokalu
-- zgadza się na zamieszkanie najemcy przy najmie okazjonalnym (art. 19a ust. 2 pkt 3 UOPL).
CREATE TRIGGER najem_reguly_rola_uczestnika
BEFORE INSERT ON sprawa_uczestnicy_role
WHEN NEW.rola NOT IN ('wynajmujacy', 'najemca', 'zamieszkujacy', 'wspolwlasciciel', 'malzonek', 'wlasciciel_innego_lokalu')
BEGIN
  SELECT RAISE(ABORT, 'rola w teczce najmu to wynajmujacy, najemca, zamieszkujacy, wspolwlasciciel, malzonek albo wlasciciel_innego_lokalu');
END;

-- Zgody najmu i zgody rdzenia przy umowie o zarządzanie z konsumentem (żądanie rozpoczęcia pracy
-- i odstąpienie, art. 15, 21 i 27 upk).
CREATE TRIGGER najem_reguly_rodzaj_zgody
BEFORE INSERT ON sprawa_zgody_wpisy
WHEN NEW.rodzaj NOT IN ('wybor_rodzaju_najmu', 'zgoda_na_naprawe', 'decyzja_o_koncu_umowy', 'zadanie_rozpoczecia_pracy', 'odstapienie')
BEGIN
  SELECT RAISE(ABORT, 'zgoda w teczce najmu to wybor_rodzaju_najmu, zgoda_na_naprawe, decyzja_o_koncu_umowy, zadanie_rozpoczecia_pracy albo odstapienie');
END;

-- Umowa pośrednictwa leży w teczce oferty albo poszukiwania i ma tu powiązanie; teczka najmu ma
-- tylko umowę o zarządzanie.
CREATE TRIGGER najem_reguly_umowa_o_zarzadzanie
BEFORE INSERT ON sprawa_umowy_wpisy
WHEN NEW.rodzaj <> 'zarzadzanie'
BEGIN
  SELECT RAISE(ABORT, 'umowa z biurem w teczce najmu to umowa o zarządzanie; pośrednictwo leży w teczce oferty albo poszukiwania');
END;
"#,
];

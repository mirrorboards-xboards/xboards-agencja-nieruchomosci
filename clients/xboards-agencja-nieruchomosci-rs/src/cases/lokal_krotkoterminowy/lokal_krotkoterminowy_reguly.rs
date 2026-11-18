//! The kind `lokal-krotkoterminowy-reguly` of the room `cases/lokal_krotkoterminowy`: its migrations, the values of its enumerated
//! columns and its operations.

/// The kind: its name, the kinds it requires and its migrations in the order of their
/// numbers.
pub const KIND: xmeta_room::Kind = xmeta_room::Kind {
    name: "lokal-krotkoterminowy-reguly",
    requires: &[
        &super::sprawa_korespondencja::KIND,
        &super::sprawa_metryka::KIND,
        &super::sprawa_protokoly::KIND,
        &super::sprawa_uczestnicy::KIND,
        &super::sprawa_umowy::KIND,
        &super::sprawa_zgody::KIND,
    ],
    migrations: &[
        r#"-- Reguły teczki lokalu krótkoterminowego na tabelach rdzenia: typ pokoju, etapy lokalu z ich
-- przejściami, role, rodzaje zgód, jedna umowa o zarządzanie i pisma, które zawsze wychodzą za
-- zgodą człowieka. Rodzaj nie ma własnych tabel i wskazuje tylko rdzeń, więc jest pierwszym
-- rodzajem typu. Reguły, które zależą od tabel typu, leżą w migracjach tamtych rodzajów, np.
-- wejście w ostatnie_rozliczenie po ostatnim pobycie (lokal-krotkoterminowy-pobyty).
--
-- Rdzeń zna etapy przyjeta, przerwana i zamknieta. Po przerwaniu przychodzi zamknieta albo powrót
-- do etapu sprzed przerwania za decyzją wznowienie_sprawy (rdzeń, T1).

-- Teczka lokalu to pokój typu lokal_krotkoterminowy; reguły tego katalogu nie pasują do innej teczki.
CREATE TRIGGER lokal_krotkoterminowy_reguly_typ_pokoju
BEFORE INSERT ON sprawa_metryka_wpis
WHEN NEW.typ_pokoju <> 'lokal_krotkoterminowy'
BEGIN
  SELECT RAISE(ABORT, 'teczka lokalu krótkoterminowego ma typ pokoju lokal_krotkoterminowy');
END;

-- Etapy idą po kolei. Umowa może się skończyć już po przygotowaniu lokalu albo w ofercie
-- (wypowiedzenie, odstąpienie), więc do koniec_umowy prowadzą trzy etapy. Z każdego etapu przed
-- zamknięciem sprawa może się przerwać; po przerwaniu przychodzi zamknięcie albo powrót do etapu
-- sprzed przerwania (sprawa_metryka_powrot_z_przerwania). Decyzje przerwania i wznowienia oraz
-- pierwszy etap (przyjeta) pilnuje rdzeń.
CREATE TRIGGER lokal_krotkoterminowy_reguly_kolejnosc_etapow
BEFORE INSERT ON sprawa_metryka_etapy
WHEN EXISTS (
    SELECT 1
    FROM sprawa_metryka_etapy
  )
  AND NOT coalesce(
    (
      SELECT
        CASE etap
          WHEN 'przyjeta' THEN NEW.etap IN ('weryfikacja_wlasciciela', 'przerwana')
          WHEN 'weryfikacja_wlasciciela' THEN NEW.etap IN ('umowa_o_zarzadzanie', 'przerwana')
          WHEN 'umowa_o_zarzadzanie' THEN NEW.etap IN ('przygotowanie_lokalu', 'przerwana')
          WHEN 'przygotowanie_lokalu' THEN NEW.etap IN ('oferta_w_kanalach', 'koniec_umowy', 'przerwana')
          WHEN 'oferta_w_kanalach' THEN NEW.etap IN ('prowadzenie_lokalu', 'koniec_umowy', 'przerwana')
          WHEN 'prowadzenie_lokalu' THEN NEW.etap IN ('koniec_umowy', 'przerwana')
          WHEN 'koniec_umowy' THEN NEW.etap IN ('zamkniecie_lokalu', 'przerwana')
          WHEN 'zamkniecie_lokalu' THEN NEW.etap IN ('ostatnie_rozliczenie', 'przerwana')
          WHEN 'ostatnie_rozliczenie' THEN NEW.etap IN ('po_zarzadzaniu', 'przerwana')
          WHEN 'po_zarzadzaniu' THEN NEW.etap IN ('zamknieta', 'przerwana')
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
    0
  )
BEGIN
  SELECT RAISE(ABORT, 'etapy lokalu idą po kolei: przyjeta, weryfikacja_wlasciciela, umowa_o_zarzadzanie, przygotowanie_lokalu, oferta_w_kanalach, prowadzenie_lokalu, koniec_umowy, zamkniecie_lokalu, ostatnie_rozliczenie, po_zarzadzaniu, zamknieta; przerwać można każdy etap przed zamknięciem, a po przerwaniu przychodzi zamknieta albo powrót do etapu sprzed przerwania');
END;

-- Etapy lokalu z profilu („Etapy”), raz od umowy o zarządzanie do jej końca. Pobyty gości mają
-- własny przebieg w etapie prowadzenie_lokalu. Stoi po regule kolejności: SQLite wywołuje najpierw
-- trigger utworzony później, więc etap spoza listy dostaje tę odmowę.
CREATE TRIGGER lokal_krotkoterminowy_reguly_etapy_lokalu
BEFORE INSERT ON sprawa_metryka_etapy
WHEN NEW.etap NOT IN (
  'przyjeta',
  'weryfikacja_wlasciciela',
  'umowa_o_zarzadzanie',
  'przygotowanie_lokalu',
  'oferta_w_kanalach',
  'prowadzenie_lokalu',
  'koniec_umowy',
  'zamkniecie_lokalu',
  'ostatnie_rozliczenie',
  'po_zarzadzaniu',
  'przerwana',
  'zamknieta'
)
BEGIN
  SELECT RAISE(ABORT, 'etap spoza etapów lokalu krótkoterminowego');
END;

-- Lokal przygotowuje się według zawartej umowy o zarządzanie: przejęcie, wyposażenie i zamek
-- robi biuro jako zarządca.
CREATE TRIGGER lokal_krotkoterminowy_reguly_przygotowanie_po_zawarciu_umowy
BEFORE INSERT ON sprawa_metryka_etapy
WHEN NEW.etap = 'przygotowanie_lokalu'
  AND NOT EXISTS (
    SELECT 1
    FROM sprawa_umowy_wpisy
    WHERE rodzaj = 'zarzadzanie'
      AND stan = 'zawarta'
  )
BEGIN
  SELECT RAISE(ABORT, 'etap przygotowanie_lokalu wymaga zawartej umowy o zarządzanie');
END;

-- Oferta w kanałach to praca według umowy: przed końcem terminu na odstąpienie konsumenta tylko
-- na jego wyraźne żądanie (art. 15 ust. 3 i art. 21 ust. 2 upk, R8), czyli od praca_od. Lokal
-- idzie do gości dopiero po spisaniu stanu przy przejęciu: protokół zwrotu porównuje się z nim.
CREATE TRIGGER lokal_krotkoterminowy_reguly_oferta_po_pracy_od_i_przejeciu
BEFORE INSERT ON sprawa_metryka_etapy
WHEN NEW.etap = 'oferta_w_kanalach'
  AND (
    NOT EXISTS (
      SELECT 1
      FROM sprawa_umowy_wpisy AS umowa
      JOIN sprawa_umowy_praca_od AS praca ON praca.umowa_id = umowa.id
      WHERE umowa.rodzaj = 'zarzadzanie'
        AND umowa.stan = 'zawarta'
        AND praca.od <= NEW.od
    )
    OR NOT EXISTS (
      SELECT 1
      FROM sprawa_protokoly_wpisy
      WHERE rodzaj = 'przejecie'
        AND stan = 'podpisany'
    )
  )
BEGIN
  SELECT RAISE(ABORT, 'etap oferta_w_kanalach wymaga zawartej umowy, od której biuro już może pracować (praca_od), i podpisanego protokołu przejęcia');
END;

-- Koniec umowy ma podstawę: datę końca, wypowiedzenie albo odstąpienie, albo zakończoną umowę. Od
-- daty końca zależy, których nocy nie wolno sprzedać. Rdzeń pozwala wpisać datę końca bez aneksu
-- po wypowiedzeniu albo odstąpieniu, ale samo zdarzenie też jest tu podstawą.
CREATE TRIGGER lokal_krotkoterminowy_reguly_koniec_umowy_z_podstawa
BEFORE INSERT ON sprawa_metryka_etapy
WHEN NEW.etap = 'koniec_umowy'
  AND NOT EXISTS (
    SELECT 1
    FROM sprawa_umowy_wpisy AS umowa
    WHERE umowa.rodzaj = 'zarzadzanie'
      AND umowa.zawarto IS NOT NULL
      AND (
        umowa.koniec IS NOT NULL
        OR umowa.stan = 'zakonczona'
        OR EXISTS (
          SELECT 1
          FROM sprawa_umowy_zdarzenia
          WHERE umowa_id = umowa.id
            AND rodzaj IN ('wypowiedzenie', 'odstapienie')
        )
      )
  )
BEGIN
  SELECT RAISE(ABORT, 'etap koniec_umowy wymaga daty końca umowy o zarządzanie, jej wypowiedzenia albo odstąpienia, albo zakończonej umowy');
END;

-- Człowiek oddaje lokal i klucze właścicielowi z protokołem zwrotu, zanim przyjdzie ostatnie
-- rozliczenie.
CREATE TRIGGER lokal_krotkoterminowy_reguly_rozliczenie_po_zwrocie_lokalu
BEFORE INSERT ON sprawa_metryka_etapy
WHEN NEW.etap = 'ostatnie_rozliczenie'
  AND NOT EXISTS (
    SELECT 1
    FROM sprawa_protokoly_wpisy
    WHERE rodzaj = 'zwrot'
      AND stan = 'podpisany'
  )
BEGIN
  SELECT RAISE(ABORT, 'etap ostatnie_rozliczenie wymaga podpisanego protokołu zwrotu lokalu');
END;

-- Role z profilu („Strony i uczestnicy”). Pełnomocnik wynika z umocowań rdzenia. Kanały,
-- wykonawcy i sąsiedzi nie są uczestnikami: kanał jest kolumną pobytu, wykonawca odnośnikiem do
-- rejestru, a sąsiad tekstem skargi.
CREATE TRIGGER lokal_krotkoterminowy_reguly_role
BEFORE INSERT ON sprawa_uczestnicy_role
WHEN NEW.rola NOT IN ('wlasciciel_lokalu', 'wspolwlasciciel', 'malzonek', 'gosc', 'zarzadca_budynku')
BEGIN
  SELECT RAISE(ABORT, 'rola spoza ról lokalu: wlasciciel_lokalu, wspolwlasciciel, malzonek, gosc, zarzadca_budynku');
END;

CREATE TRIGGER lokal_krotkoterminowy_reguly_zmieniona_rola
BEFORE UPDATE OF rola ON sprawa_uczestnicy_role
WHEN NEW.rola NOT IN ('wlasciciel_lokalu', 'wspolwlasciciel', 'malzonek', 'gosc', 'zarzadca_budynku')
BEGIN
  SELECT RAISE(ABORT, 'rola spoza ról lokalu: wlasciciel_lokalu, wspolwlasciciel, malzonek, gosc, zarzadca_budynku');
END;

-- Zgody właściciela z profilu i zgody rdzenia przy umowie z biurem. Zgody się nie zmienia (rdzeń),
-- więc wystarczy reguła przy wpisie.
CREATE TRIGGER lokal_krotkoterminowy_reguly_zgody
BEFORE INSERT ON sprawa_zgody_wpisy
WHEN NEW.rodzaj NOT IN (
  'zgoda_na_naprawe',
  'zgoda_na_wyposazenie',
  'noce_wlasciciela',
  'zgoda_na_kanal',
  'zadanie_rozpoczecia_pracy',
  'odstapienie'
)
BEGIN
  SELECT RAISE(ABORT, 'zgoda spoza zgód lokalu: zgoda_na_naprawe, zgoda_na_wyposazenie, noce_wlasciciela, zgoda_na_kanal, zadanie_rozpoczecia_pracy, odstapienie');
END;

-- Biuro prowadzi lokal na podstawie umowy o zarządzanie (R6). Umowa pośrednictwa leży w teczce
-- oferty albo poszukiwania.
CREATE TRIGGER lokal_krotkoterminowy_reguly_umowa_o_zarzadzanie
BEFORE INSERT ON sprawa_umowy_wpisy
WHEN NEW.rodzaj <> 'zarzadzanie'
BEGIN
  SELECT RAISE(ABORT, 'teczka lokalu ma tylko umowę o zarządzanie');
END;

-- Teczka prowadzi jeden lokal od jednej umowy do jej końca; zmiany warunków idą aneksem, a nowa
-- umowa to nowa teczka. Dzięki temu reguły typu czytają warunki jednej umowy.
CREATE TRIGGER lokal_krotkoterminowy_reguly_jedna_zawarta_umowa
BEFORE UPDATE OF stan ON sprawa_umowy_wpisy
WHEN NEW.stan = 'zawarta'
  AND OLD.stan <> 'zawarta'
  AND EXISTS (
    SELECT 1
    FROM sprawa_umowy_wpisy
    WHERE id <> NEW.id
      AND zawarto IS NOT NULL
  )
BEGIN
  SELECT RAISE(ABORT, 'teczka lokalu ma najwyżej jedną zawartą umowę o zarządzanie; nowa umowa to nowa teczka');
END;

-- Każde pismo do wspólnoty, spółdzielni i zarządcy budynku wychodzi za zgodą człowieka
-- („Sprawy za zgodą”), także pismo według wzoru i pismo do uczestnika z rolą zarzadca_budynku.
CREATE TRIGGER lokal_krotkoterminowy_reguly_pismo_do_wspolnoty_za_zgoda
BEFORE UPDATE OF stan ON sprawa_korespondencja_wysylki
WHEN NEW.stan = 'wyslana'
  AND OLD.stan = 'projekt'
  AND (
    NEW.adresat_rodzaj IN ('wspolnota', 'spoldzielnia', 'zarzadca')
    OR EXISTS (
      SELECT 1
      FROM sprawa_uczestnicy_role
      WHERE uczestnik_id = NEW.adresat_uczestnik_id
        AND rola = 'zarzadca_budynku'
    )
  )
BEGIN
  SELECT RAISE(ABORT, 'pismo do wspólnoty, spółdzielni albo zarządcy budynku przed wysłaniem czeka na zgodę');
END;
"#,
    ],
};

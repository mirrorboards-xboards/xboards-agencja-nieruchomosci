-- Warunki, na jakich klient wynajmie nieruchomość, w wersjach. Czynsz leży w oferta-ceny
-- (dotyczy = 'najem'). Pisze Opiekun oferty; każdą wersję zatwierdza klient zgodą warunki_najmu.
-- Kwoty w groszach.
--
-- Rodzaj umowy najmu wybiera klient; Opiekun pokazuje mu różnice ze wzorów biura i nie doradza.
-- Granice kaucji z ustawy (art. 6 ust. 1, art. 19a ust. 4 i art. 19f ust. 5 UOPL) dotyczą umowy
-- najmu, więc pilnuje ich teczka najmu.
CREATE TABLE oferta_najem_warunki (
  id TEXT PRIMARY KEY,
  -- Opłaty, które najemca płaci obok czynszu, i sposób rozliczenia mediów.
  oplaty_obok_czynszu TEXT,
  rozliczenie_mediow TEXT,
  kaucja INTEGER CHECK (kaucja >= 0),
  -- Rodzaje umowy najmu, które klient dopuszcza.
  dopuszcza_zwykly INTEGER NOT NULL CHECK (dopuszcza_zwykly IN (0, 1)),
  dopuszcza_okazjonalny INTEGER NOT NULL CHECK (dopuszcza_okazjonalny IN (0, 1)),
  dopuszcza_instytucjonalny INTEGER NOT NULL CHECK (dopuszcza_instytucjonalny IN (0, 1)),
  -- Od kiedy lokal jest wolny oraz najkrótszy i najdłuższy okres najmu, np. „12 miesięcy”.
  wolny_od TEXT,
  okres_min TEXT,
  okres_max TEXT,
  wyposazenie TEXT,
  liczba_osob INTEGER CHECK (liczba_osob > 0),
  zwierzeta TEXT,
  palenie TEXT,
  -- Czy biuro ma po zawarciu umowy prowadzić najem; wtedy dochodzi umowa o zarządzanie (R6).
  prowadzenie_najmu INTEGER NOT NULL CHECK (prowadzenie_najmu IN (0, 1)),
  zgoda_id TEXT NOT NULL REFERENCES sprawa_zgody_wpisy (id),
  zastepuje_id TEXT REFERENCES oferta_najem_warunki (id),

  CONSTRAINT dopuszcza_rodzaj_najmu CHECK (dopuszcza_zwykly + dopuszcza_okazjonalny + dopuszcza_instytucjonalny >= 1),
  CONSTRAINT nie_zastepuje_samej_siebie CHECK (zastepuje_id IS NOT id)
) STRICT;

-- Warunki najmu zatwierdza klient: zgoda warunki_najmu uczestnika z obowiązującą rolą klient,
-- która dotyczy tej wersji.
CREATE TRIGGER oferta_najem_warunki_ze_zgoda_klienta
BEFORE INSERT ON oferta_najem_warunki
WHEN NOT EXISTS (
  SELECT 1
  FROM sprawa_zgody_wpisy AS zgoda
  JOIN sprawa_uczestnicy_role AS rola ON rola.uczestnik_id = zgoda.uczestnik_id
  WHERE zgoda.id = NEW.zgoda_id
    AND zgoda.rodzaj = 'warunki_najmu'
    AND zgoda.dotyczy_tabela = 'oferta_najem_warunki'
    AND zgoda.dotyczy_id = NEW.id
    AND rola.rola = 'klient'
    AND rola.zakonczono IS NULL
)
BEGIN
  SELECT RAISE(ABORT, 'warunki najmu mają zgodę warunki_najmu klienta, która ich dotyczy');
END;

-- Pierwsza wersja nie zastępuje żadnej; następna zastępuje wersję, która obowiązuje.
CREATE TRIGGER oferta_najem_warunki_zastepuja_obowiazujace
BEFORE INSERT ON oferta_najem_warunki
WHEN (
    NEW.zastepuje_id IS NULL
    AND EXISTS (
      SELECT 1
      FROM oferta_najem_warunki
    )
  )
  OR (
    NEW.zastepuje_id IS NOT NULL
    AND NOT EXISTS (
      SELECT 1
      FROM oferta_najem_warunki AS poprzednie
      WHERE poprzednie.id = NEW.zastepuje_id
        AND NOT EXISTS (
          SELECT 1
          FROM oferta_najem_warunki AS nowsze
          WHERE nowsze.zastepuje_id = poprzednie.id
        )
    )
  )
BEGIN
  SELECT RAISE(ABORT, 'nowe warunki najmu zastępują warunki, które obowiązują');
END;

CREATE TRIGGER oferta_najem_warunki_bez_zmian
BEFORE UPDATE ON oferta_najem_warunki
BEGIN
  SELECT RAISE(ABORT, 'warunków najmu się nie zmienia; nowa wersja zastępuje poprzednią');
END;

-- INSERT OR REPLACE i upsert (ON CONFLICT DO UPDATE) zastąpiłyby istniejący wiersz z ominięciem
-- reguł zmian i blokady usuwania, bo pokój nie włącza recursive_triggers.
CREATE TRIGGER oferta_najem_warunki_bez_zastepowania
BEFORE INSERT ON oferta_najem_warunki
WHEN EXISTS (
  SELECT 1
  FROM oferta_najem_warunki
  WHERE id = NEW.id
)
BEGIN
  SELECT RAISE(ABORT, 'wiersza teczki nie zastępuje się nowym wpisem; zmiana idzie przez UPDATE');
END;

CREATE TRIGGER oferta_najem_warunki_bez_usuwania
BEFORE DELETE ON oferta_najem_warunki
BEGIN
  SELECT RAISE(ABORT, 'wierszy teczki się nie usuwa (R9); pomyłkę poprawia stan albo nowy wiersz');
END;

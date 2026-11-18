-- name: metryka :one
-- Metryka pokoju: transakcja, czynność, kancelaria, notariusz i nieruchomość.
SELECT *
FROM wymiana_metryka_wpis
WHERE id = 1;

-- name: otworz_pokoj_wymiany :exec
-- Biuro zakłada pokój za zgodą człowieka, który przyjął transakcję, i zaprasza kancelarię.
INSERT INTO wymiana_metryka_wpis (id, oznaczenie, transakcja_odnosnik, czynnosc, kancelaria_nazwa,
  kancelaria_adres, notariusz, nieruchomosc_adres, numer_kw, otwarto)
VALUES (1, :oznaczenie, :transakcja_odnosnik, :czynnosc, :kancelaria_nazwa, :kancelaria_adres,
  :notariusz, :nieruchomosc_adres, :numer_kw, :otwarto);

-- name: podaj_sygnature_kancelarii :exec
-- Kancelaria podaje sygnaturę swojej sprawy; robi to raz.
-- param sygnatura_kancelarii: string
UPDATE wymiana_metryka_wpis
SET sygnatura_kancelarii = :sygnatura_kancelarii
WHERE id = 1;

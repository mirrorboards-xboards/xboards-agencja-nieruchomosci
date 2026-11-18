-- name: terminy :many
-- Terminy sprawy, najbliższy pierwszy.
SELECT *
FROM sprawa_terminy_wpisy
ORDER BY koniec, id;

-- name: terminy_biegnace :many
-- Terminy, które jeszcze biegną, najbliższy pierwszy.
SELECT *
FROM sprawa_terminy_wpisy
WHERE stan = 'biegnie'
ORDER BY koniec, id;

-- name: wpisz_termin :exec
-- Termin wyliczony programem albo umówiony; biegnie od zdarzenia.
INSERT INTO sprawa_terminy_wpisy (id, rodzaj, co, zdarzenie_tabela, zdarzenie_id, zdarzenie_chwila,
  koniec, podstawa, pilnuje_stanowisko, kalendarz_odnosnik, stan)
VALUES (:id, :rodzaj, :co, :zdarzenie_tabela, :zdarzenie_id, :zdarzenie_chwila, :koniec, :podstawa,
  :pilnuje_stanowisko, :kalendarz_odnosnik, 'biegnie');

-- name: przelicz_termin :exec
-- Nowy koniec niepotwierdzonego terminu, np. po zmianie zdarzenia. Potwierdzonego terminu się nie
-- przelicza: przesunięty akt to nowy termin z nowym potwierdzeniem, a stary staje się zbędny.
UPDATE sprawa_terminy_wpisy
SET zdarzenie_tabela = :zdarzenie_tabela,
  zdarzenie_id = :zdarzenie_id,
  zdarzenie_chwila = :zdarzenie_chwila,
  koniec = :koniec,
  kalendarz_odnosnik = :kalendarz_odnosnik
WHERE id = :id;

-- name: zmien_stan_terminu :exec
-- Termin dotrzymany, który minął albo przestał być potrzebny; minięty trafia do raportu agenta.
UPDATE sprawa_terminy_wpisy
SET stan = :stan
WHERE id = :id;

-- name: odnotuj_potwierdzenie_terminu :exec
-- Umówiony termin potwierdzony zatwierdzoną decyzją potwierdzenie_terminu.
-- param potwierdzenie_decyzja_id: string
UPDATE sprawa_terminy_wpisy
SET potwierdzenie_decyzja_id = :potwierdzenie_decyzja_id
WHERE id = :id;

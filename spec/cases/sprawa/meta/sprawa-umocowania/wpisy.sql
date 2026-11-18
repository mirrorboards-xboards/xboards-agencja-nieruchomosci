-- name: umocowania :many
-- Pełnomocnictwa i reprezentacje w teczce, w kolejności wpisania.
SELECT *
FROM sprawa_umocowania_wpisy
ORDER BY rowid;

-- name: wpisz_umocowanie :exec
-- Pełnomocnictwo albo reprezentacja podmiotu, z dokumentem, na którym stoi.
INSERT INTO sprawa_umocowania_wpisy (id, rodzaj, mocodawca_uczestnik_id, umocowany_uczestnik_id,
  umocowane_biuro, zakres, forma, sposob_reprezentacji, dokument_id, udzielono, wazne_do)
VALUES (:id, :rodzaj, :mocodawca_uczestnik_id, :umocowany_uczestnik_id, :umocowane_biuro, :zakres,
  :forma, :sposob_reprezentacji, :dokument_id, :udzielono, :wazne_do);

-- name: zmien_dane_umocowania :exec
-- Poprawka umocowania; podnosi jego wersję, więc Kontroler sprawdza je od nowa. Data udzielenia
-- się nie zmienia, a wygaśnięcie wpisuje odnotuj_wygasniecie_umocowania.
UPDATE sprawa_umocowania_wpisy
SET rodzaj = :rodzaj,
  mocodawca_uczestnik_id = :mocodawca_uczestnik_id,
  umocowany_uczestnik_id = :umocowany_uczestnik_id,
  umocowane_biuro = :umocowane_biuro,
  zakres = :zakres,
  forma = :forma,
  sposob_reprezentacji = :sposob_reprezentacji,
  dokument_id = :dokument_id,
  wazne_do = :wazne_do
WHERE id = :id;

-- name: odnotuj_wygasniecie_umocowania :exec
-- Odwołanie albo wygaśnięcie umocowania, z datą i źródłem wiadomości; wpisuje się je raz.
-- param wygaslo: string
-- param wygasniecie_powod: "odwolanie" | "smierc_mocodawcy" | "smierc_umocowanego" | "uplyw_terminu" | "wykreslenie_z_rejestru" | "inne"
-- param wygasniecie_zrodlo: string
UPDATE sprawa_umocowania_wpisy
SET wygaslo = :wygaslo,
  wygasniecie_powod = :wygasniecie_powod,
  wygasniecie_zrodlo = :wygasniecie_zrodlo
WHERE id = :id;

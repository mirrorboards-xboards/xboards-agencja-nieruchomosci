-- name: usterki :many
-- Zgłoszenia usterek, najnowsze pierwsze.
SELECT *
FROM najem_usterki_zgloszenia
ORDER BY chwila DESC, id;

-- name: zglos_usterke :exec
-- Zgłoszenie od najemcy, wprost albo przez Recepcję, z propozycją, kogo obciąża naprawa;
-- identyfikator (UUID) nadaje aplikacja, bo wskazują go decyzje i zgoda wynajmującego.
INSERT INTO najem_usterki_zgloszenia (id, chwila, zglosil_uczestnik_id, wplyw_id, opis,
  zdjecia_sciezka, zdjecia_skrot, uniemozliwia, obciaza)
VALUES (:id, :chwila, :zglosil_uczestnik_id, :wplyw_id, :opis, :zdjecia_sciezka, :zdjecia_skrot,
  :uniemozliwia, :obciaza);

-- name: ustal_kogo_obciaza_usterka :exec
-- Kogo obciąża naprawa; spór rozstrzyga decyzja człowieka.
-- param obciaza: "wynajmujacy" | "najemca" | "spor"
UPDATE najem_usterki_zgloszenia
SET obciaza = :obciaza,
  spor_decyzja_id = :spor_decyzja_id
WHERE id = :id;

-- name: zlec_naprawe_usterki :exec
-- Człowiek z biura zleca naprawę obciążającą wynajmującego: wykonawca i koszt, a ponad kwotę
-- napraw także zgoda wynajmującego i decyzja.
-- param koszt: integer
-- param zlecil: string
-- param zlecono: string
-- param wykonawca: string
UPDATE najem_usterki_zgloszenia
SET koszt = :koszt,
  zlecil = :zlecil,
  zlecono = :zlecono,
  wykonawca = :wykonawca,
  zgoda_wynajmujacego_id = :zgoda_wynajmujacego_id,
  zlecenie_decyzja_id = :zlecenie_decyzja_id
WHERE id = :id;

-- name: odnotuj_usuniecie_usterki :exec
-- Usterka usunięta, co najemca potwierdził wiadomością.
-- param usunieto: string
-- param potwierdzenie_wplyw_id: string
UPDATE najem_usterki_zgloszenia
SET usunieto = :usunieto,
  potwierdzenie_wplyw_id = :potwierdzenie_wplyw_id
WHERE id = :id;

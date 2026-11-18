-- name: rozeslania_polisy :many
-- Umowy, które dostały kopię jednej polisy, od najwcześniejszego wysłania.
SELECT *
FROM kierownik_dzialu_polisy_rozeslania
WHERE polisa_numer = :polisa_numer
ORDER BY rozeslano, id;

-- name: odnotuj_rozeslanie_polisy :exec
-- Agent wysłał klientowi kopię polisy i podał to w raporcie.
INSERT INTO kierownik_dzialu_polisy_rozeslania (polisa_numer, polisa_odnosnik, umowa_odnosnik, raport_id,
  rozeslano)
VALUES (:polisa_numer, :polisa_odnosnik, :umowa_odnosnik, :raport_id, :rozeslano);

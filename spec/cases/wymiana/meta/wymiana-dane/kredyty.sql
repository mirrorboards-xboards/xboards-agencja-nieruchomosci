-- name: kredyty :many
-- Kredyt kupującego w tym zakresie, którego potrzebuje akt.
SELECT kredyt.*, wpis.autor, wpis.chwila, wpis.poprawia_id
FROM wymiana_dane_kredyty AS kredyt
JOIN wymiana_wpisy_wpisy AS wpis ON wpis.id = kredyt.wpis_id
ORDER BY wpis.rowid;

-- name: przekaz_kredyt :exec
-- Biuro przekazuje w imieniu kupującego bank, kwotę z decyzji i warunki wypłaty, gdy kancelaria
-- o nie prosi.
INSERT INTO wymiana_wpisy_wpisy (id, strona, autor, rodzaj, chwila, odpowiada_na_id, poprawia_id)
VALUES (:wpis_id, :strona, :autor, 'kredyt', :chwila, :odpowiada_na_id, :poprawia_id);

INSERT INTO wymiana_dane_kredyty (wpis_id, strona_id, bank, kwota, warunki_wyplaty)
VALUES (:wpis_id, :strona_id, :bank, :kwota, :warunki_wyplaty);

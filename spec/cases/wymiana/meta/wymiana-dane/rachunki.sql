-- name: rachunki_do_aktu :many
-- Rachunki obu stron pokoju z wpisami; od biura z decyzją, od kancelarii jako informacja.
SELECT rachunek.*, wpis.strona, wpis.autor, wpis.chwila, wpis.poprawia_id, wpis.tresc
FROM wymiana_dane_rachunki AS rachunek
JOIN wymiana_wpisy_wpisy AS wpis ON wpis.id = rachunek.wpis_id
ORDER BY wpis.rowid;

-- name: podaj_rachunek :exec
-- Biuro przekazuje rachunek zatwierdzony przez człowieka; kancelaria podaje rachunek jako informację.
INSERT INTO wymiana_wpisy_wpisy (id, strona, autor, rodzaj, chwila, odpowiada_na_id, poprawia_id,
  tresc)
VALUES (:wpis_id, :strona, :autor, 'rachunek', :chwila, :odpowiada_na_id, :poprawia_id, :tresc);

INSERT INTO wymiana_dane_rachunki (wpis_id, numer, posiadacz, cel, decyzja_odnosnik)
VALUES (:wpis_id, :numer, :posiadacz, :cel, :decyzja_odnosnik);

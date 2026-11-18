-- name: pierwokupy :many
-- Wiadomości kancelarii o pierwokupie: doręczenie zawiadomienia i wynik, od najstarszej.
SELECT pierwokup.*, wpis.chwila, wpis.poprawia_id, wpis.tresc
FROM wymiana_akt_pierwokupy AS pierwokup
JOIN wymiana_wpisy_wpisy AS wpis ON wpis.id = pierwokup.wpis_id
ORDER BY wpis.rowid;

-- name: podaj_pierwokup :exec
-- Kancelaria podaje, kiedy uprawniony dostał zawiadomienie, a potem wynik; wynik poprawia
-- wcześniejszy wpis.
INSERT INTO wymiana_wpisy_wpisy (id, strona, autor, rodzaj, chwila, poprawia_id, tresc)
VALUES (:wpis_id, :strona, :autor, 'pierwokup', :chwila, :poprawia_id, :tresc);

INSERT INTO wymiana_akt_pierwokupy (wpis_id, uprawniony, zawiadomienie_doreczono, wynik, data)
VALUES (:wpis_id, :uprawniony, :zawiadomienie_doreczono, :wynik, :data);

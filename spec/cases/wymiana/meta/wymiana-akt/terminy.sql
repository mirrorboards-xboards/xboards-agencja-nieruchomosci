-- name: terminy_aktu :many
-- Terminy aktu z bieżącym stanem i tym, która strona już je potwierdziła.
-- column potwierdzila_biuro: boolean
-- column potwierdzila_kancelaria: boolean
-- column stan: "proponowany" | "potwierdzony" | "przesuniety" | "odwolany" | "odbyl_sie"
SELECT *
FROM wymiana_akt_terminy_biezace
ORDER BY dzien, godzina;

-- name: stany_terminow :many
-- Ogłoszenia stanów terminów obu stron, z decyzją przy potwierdzeniu biura.
SELECT stan.*, wpis.strona, wpis.autor, wpis.chwila, wpis.tresc
FROM wymiana_akt_terminy_stany AS stan
JOIN wymiana_wpisy_wpisy AS wpis ON wpis.id = stan.wpis_id
ORDER BY wpis.rowid;

-- name: zaproponuj_termin_aktu :exec
-- Strona proponuje dzień, godzinę i miejsce aktu; termin czeka na potwierdzenie obu stron.
INSERT INTO wymiana_wpisy_wpisy (id, strona, autor, rodzaj, chwila, odpowiada_na_id, tresc)
VALUES (:wpis_id, :strona, :autor, 'propozycja_terminu', :chwila, :odpowiada_na_id, :tresc);

INSERT INTO wymiana_akt_terminy (id, wpis_id, dzien, godzina, miejsce, czynnosc)
VALUES (:id, :wpis_id, :dzien, :godzina, :miejsce, :czynnosc);

-- name: oglos_stan_terminu :exec
-- Strona potwierdza, przesuwa albo odwołuje termin, a kancelaria ogłasza, że akt się odbył.
-- Biuro potwierdza termin tylko z decyzją potwierdzenie_terminu z teczki.
INSERT INTO wymiana_wpisy_wpisy (id, strona, autor, rodzaj, chwila, odpowiada_na_id, tresc)
VALUES (:wpis_id, :strona, :autor, 'stan_terminu', :chwila, :odpowiada_na_id, :tresc);

INSERT INTO wymiana_akt_terminy_stany (wpis_id, termin_id, stan, decyzja_odnosnik)
VALUES (:wpis_id, :termin_id, :stan, :decyzja_odnosnik);

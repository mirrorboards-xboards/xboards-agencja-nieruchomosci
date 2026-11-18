-- name: listy_dokumentow :many
-- Pozycje wszystkich wersji listy dokumentów kancelarii, z chwilą wpisu.
SELECT lista.*, wpis.chwila
FROM wymiana_dokumenty_lista AS lista
JOIN wymiana_wpisy_wpisy AS wpis ON wpis.id = lista.wpis_id
ORDER BY wpis.rowid, lista.rowid;

-- name: biezaca_lista_dokumentow :many
-- Pozycje najnowszej listy dokumentów; zmiana listy jest nowym wpisem z całą listą.
SELECT *
FROM wymiana_dokumenty_lista
WHERE wpis_id = (
  SELECT id
  FROM wymiana_wpisy_wpisy
  WHERE rodzaj = 'lista_dokumentow'
  ORDER BY rowid DESC
  LIMIT 1
)
ORDER BY rowid;

-- name: podaj_liste_dokumentow :exec
-- Kancelaria zaczyna listę dokumentów do aktu; pozycje dopisuje dopisz_pozycje_listy w tej samej
-- transakcji (useMetaTransaction). Nowa lista poprawia poprzednią.
INSERT INTO wymiana_wpisy_wpisy (id, strona, autor, rodzaj, chwila, poprawia_id, tresc)
VALUES (:id, :strona, :autor, 'lista_dokumentow', :chwila, :poprawia_id, :tresc);

-- name: dopisz_pozycje_listy :exec
-- Pozycja listy: dokument, od której strony, w jakiej postaci, jak świeży i do kiedy.
INSERT INTO wymiana_dokumenty_lista (wpis_id, pozycja_id, nazwa, od_strony_id, postac,
  swiezosc_dni, do_kiedy)
VALUES (:wpis_id, :pozycja_id, :nazwa, :od_strony_id, :postac, :swiezosc_dni, :do_kiedy);

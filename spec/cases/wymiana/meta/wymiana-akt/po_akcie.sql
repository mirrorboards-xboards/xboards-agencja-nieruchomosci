-- name: informacje_po_akcie :many
-- To, co kancelaria przekazała po akcie: sporządzenie aktu i złożenie wniosku do księgi.
SELECT po_akcie.*, wpis.chwila, wpis.poprawia_id, wpis.tresc
FROM wymiana_akt_po_akcie AS po_akcie
JOIN wymiana_wpisy_wpisy AS wpis ON wpis.id = po_akcie.wpis_id
ORDER BY wpis.rowid;

-- name: podaj_informacje_po_akcie :exec
-- Kancelaria przekazuje, że akt sporządzono i kiedy złożyła wniosek do księgi wieczystej.
INSERT INTO wymiana_wpisy_wpisy (id, strona, autor, rodzaj, chwila, poprawia_id, tresc)
VALUES (:wpis_id, :strona, :autor, 'po_akcie', :chwila, :poprawia_id, :tresc);

INSERT INTO wymiana_akt_po_akcie (wpis_id, akt_sporzadzono, wniosek_kw_zlozono)
VALUES (:wpis_id, :akt_sporzadzono, :wniosek_kw_zlozono);

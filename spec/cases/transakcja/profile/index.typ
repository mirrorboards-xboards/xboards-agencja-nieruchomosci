#import "/mirrorboards-core/mirrorboards-xmeta/xmeta-typ/lib.typ": dokument, part

#show: dokument.with(produkt: "XEstates", tytul: [Transakcja], podtytul: [Pokój sprawy · sprzedaż nieruchomości])

#part[Sprawa]
#include "chapters/A_sprawa/01_opis.typ"
#include "chapters/A_sprawa/02_podstawa_prawna.typ"
#include "chapters/A_sprawa/03_strony_i_uczestnicy.typ"

#part[Przebieg]
#include "chapters/B_przebieg/01_dokumenty.typ"
#include "chapters/B_przebieg/02_etapy.typ"
#include "chapters/B_przebieg/03_umowy.typ"
#include "chapters/B_przebieg/04_przekazanie.typ"
#include "chapters/B_przebieg/05_terminy.typ"

#part[Granice]
#include "chapters/C_granice/01_sprawy_za_zgoda_i_zakazane.typ"

#part[Odmiany]
#include "chapters/D_odmiany/01_odmiany_i_serie.typ"

// Księga xboardu: profile wszystkich pozycji ze spec/, najpierw pozycja całego produktu.
// Pisze ją `xmeta book`; nie poprawiaj ręcznie, tylko uruchom polecenie po zmianie pozycji.
#import "/mirrorboards-core/mirrorboards-xmeta/xmeta-typ/lib.typ": grupa, ksiega

#show: ksiega

#include "spec/system/profile/index.typ"

#grupa[Sprawy]
#include "spec/cases/sprawa/profile/index.typ"
#include "spec/cases/lokal_krotkoterminowy/profile/index.typ"
#include "spec/cases/najem/profile/index.typ"
#include "spec/cases/oferta/profile/index.typ"
#include "spec/cases/poszukiwanie/profile/index.typ"
#include "spec/cases/transakcja/profile/index.typ"
#include "spec/cases/wymiana/profile/index.typ"

#grupa[Stanowiska]
#include "spec/positions/biuro/kierownik_biura/profile/index.typ"
#include "spec/positions/biuro/recepcja/profile/index.typ"
#include "spec/positions/kontrola/kierownik_kontroli/profile/index.typ"
#include "spec/positions/kontrola/kontroler/profile/index.typ"
#include "spec/positions/najem/kierownik_najmu/profile/index.typ"
#include "spec/positions/najem/konsjerz/profile/index.typ"
#include "spec/positions/najem/koordynator_operacji/profile/index.typ"
#include "spec/positions/najem/opiekun_najmu/profile/index.typ"
#include "spec/positions/najem/zarzadca_obiektu/profile/index.typ"
#include "spec/positions/posrednictwo/kierownik_posrednictwa/profile/index.typ"
#include "spec/positions/posrednictwo/opiekun_oferty/profile/index.typ"
#include "spec/positions/posrednictwo/opiekun_poszukujacego/profile/index.typ"
#include "spec/positions/transakcje/analityk_kredytowy/profile/index.typ"
#include "spec/positions/transakcje/analityk_stanu_prawnego/profile/index.typ"
#include "spec/positions/transakcje/kierownik_transakcji/profile/index.typ"
#include "spec/positions/transakcje/koordynator_transakcji/profile/index.typ"

#grupa[Rejestry]
#include "spec/registers/katalog_ofert/profile/index.typ"
#include "spec/registers/klienci/profile/index.typ"
#include "spec/registers/rejestr_kontroli/profile/index.typ"
#include "spec/registers/wykonawcy/profile/index.typ"

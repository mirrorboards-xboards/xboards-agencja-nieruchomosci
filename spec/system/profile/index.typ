#import "/mirrorboards-core/mirrorboards-xmeta/xmeta-typ/lib.typ": dokument, part

#show: dokument.with(produkt: "XEstates", tytul: [Agencja nieruchomości], podtytul: [Agenci agencji nieruchomości])

#part[Układ]
#include "chapters/A_uklad/01_wprowadzenie.typ"
#include "chapters/A_uklad/02_zasady.typ"
#include "chapters/A_uklad/03_dzialy_i_stanowiska.typ"

#part[Praca]
#include "chapters/B_praca/01_kierownik_dzialu.typ"
#include "chapters/B_praca/02_agent_sprawy.typ"

#part[Dane i granice]
#include "chapters/C_dane_i_granice/01_pokoje_i_dostep.typ"
#include "chapters/C_dane_i_granice/02_katalogi_danych.typ"
#include "chapters/C_dane_i_granice/03_granice_dzialania.typ"

#part[Otoczenie]
#include "chapters/D_otoczenie/01_akty_prawne.typ"
#include "chapters/D_otoczenie/02_rozstrzygniecia.typ"

#part[Sprawy otwarte]

+ Obowiązki najmu krótkoterminowego, które trzeba zbadać w przepisach:
  rejestracja lokalu, opłata miejscowa, meldunek gości i dane dla platform.
  Do tego: czy dane o czynszach dla gminy obejmują pobyty
  krótkoterminowe#footnote[Art. 186a ugn.] i jak informować gości o ich
  danych, które biuro dostaje z kanałów sprzedaży#footnote[Art. 14 RODO.].
+ Czy ubezpieczenie odpowiedzialności cywilnej pośrednika obejmuje szkody
  wyrządzone pracą agentów. Ustawa mówi o osobach, które działają pod
  nadzorem pośrednika#footnote[Art. 181 ust. 3 ugn.].
+ Zakres ustawy o przeciwdziałaniu praniu pieniędzy: ustawa obejmuje
  pośredników#footnote[Art. 2 ust. 1 pkt 18 AML.], a nie zarządców
  nieruchomości. Do ustalenia, które obowiązki biuro ma w sprawach, w których
  jest tylko zarządcą, którego dnia liczy kurs euro przy progu czynszu
  10 000 euro#footnote[Art. 5 AML.] i czy pośrednik, który pracuje pod marką
  biura jako osobny przedsiębiorca, jest osobną instytucją obowiązaną.
+ Dane osobowe w pokoju wymiany: czy biuro i kancelaria notarialna są
  współadministratorami danych stron#footnote[Art. 26 RODO.].
+ Kto jest właścicielem agentów każdego stanowiska.
+ Rozstrzygnięcie R9 a prawo do usunięcia danych: osoba może żądać usunięcia
  swoich danych, np. gdy nie są już potrzebne albo gdy cofnęła
  zgodę#footnote[Art. 17 ust. 1 RODO.]. Biuro niczego nie usuwa, więc takiego
  żądania nie wykona. Prawo do usunięcia nie obejmuje danych, które biuro
  musi przetwarzać z mocy ustawy, ani danych potrzebnych do
  roszczeń#footnote[Art. 17 ust. 3 lit. b i e RODO.]. Przed pierwszym
  prawdziwym klientem biuro zastąpi R9 zasadami usuwania danych.
+ Informacja o zamrożeniu wartości majątkowych: biuro przekazuje ją
  Generalnemu Inspektorowi najpóźniej w 2 dni robocze od
  zamrożenia#footnote[Art. 119 ust. 2 AML.]. Rejestr kontroli zapisuje przy
  sprawdzeniu list sankcyjnych tylko chwilę zamrożenia i chwilę przekazania
  informacji. Informacji nie ma wśród rodzajów zawiadomień, więc baza nie
  zna jej projektu, decyzji o przekazaniu ani terminu. Do rozstrzygnięcia,
  czy informacja ma być w bazie osobnym rodzajem wpisu.
+ Reguły po stronie aplikacji: części zasad z profili baza nie pilnuje, np.
  tego, że tylko Kontrola przekazuje prośby wprost właścicielowi biura.
  Wymienia je rozdział „Katalogi danych” w części „Czego baza nie pilnuje”.

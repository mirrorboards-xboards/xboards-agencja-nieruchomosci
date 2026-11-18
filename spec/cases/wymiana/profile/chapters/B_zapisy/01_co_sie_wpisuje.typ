= Co się wpisuje

Katalog danych pokoju leży w xboardzie agencji, w `cases/wymiana/meta`.
XNotariusz pisze według tego samego katalogu, więc obie strony piszą do
tych samych tabel (dokument „Agencja nieruchomości”, rozdział „Katalogi danych”).

== Zasady wpisów

/ Autor i strona: każdy wpis ma autora, stronę pokoju (biuro albo
  kancelaria), czas i podpis autora jego kluczem. Stronę deklaruje piszący.
  Aplikacja porównuje ją z organizacją autora, którą poświadcza serwer
  pokoju. Baza tego porównania nie robi.
/ Bez zmian: wpisu się nie zmienia. Poprawka jest nowym wpisem, który
  wskazuje poprzedni. Nowa wersja projektu aktu nie zastępuje starej, tylko
  staje obok niej z numerem wersji.
/ Pliki: plik leży w repozytorium plików pokoju. Wpis trzyma ścieżkę i skrót
  treści. Potwierdzenie albo uwaga, która dotyczy pliku, obejmuje jego skrót.
/ Odpowiedź: wpis, który odpowiada na inny, wskazuje go. Dzięki temu widać, że
  uwaga dotyczy wersji 3 projektu, a nie wersji 2.

== Metryka pokoju

/ Transakcja: oznaczenie transakcji w biurze, np. „Transakcja 2026/017,
  ul. Polna 7 m. 12”.
/ Kancelaria i notariusz: nazwa i adres kancelarii, imię i nazwisko
  notariusza, a gdy kancelaria ją poda, jej sygnatura sprawy.
/ Nieruchomość: adres i numer księgi wieczystej.
/ Strony: imiona i nazwiska albo nazwy stron i to, którą stronę biuro
  reprezentuje.
/ Czynność: np. umowa przedwstępna w formie aktu, umowa sprzedaży pod
  warunkiem, umowa przenosząca własność albo oświadczenie najemcy.
/ Stan: otwarty albo zamknięty, z datami.

== Od biura

/ Pełnomocnictwa: pełnomocnictwo każdej strony, w której imieniu pisze
  Koordynator, z zakresem. Gdy strony chcą, żeby biuro dostawało projekt aktu
  i informację o akcie, w pokoju leży ich zgoda na to (rozdział „Podstawa
  prawna”).
/ Dane stron do aktu: dane, które akt podaje o osobach, które w nim
  uczestniczą#footnote[Art. 92 § 1 pkt 4 PoN.], i inne dane, o które prosi
  kancelaria, np. stan cywilny. Każda dana wskazuje dokument, z którego
  pochodzi (dokument „Agencja nieruchomości”, zasada „Każda wartość ma źródło”).
/ Dane nieruchomości: oznaczenie z odpisu księgi wieczystej i z ewidencji
  gruntów i budynków.
/ Dokumenty: każdy dokument z listy kancelarii z wystawcą, datą wystawienia,
  ważnością i plikiem. Wpis mówi też, czy oryginał będzie w dniu aktu i kto
  go przyniesie.
/ Propozycje terminu: dni i godziny, które pasują stronom.
/ Uwagi stron do projektu: każda uwaga wskazuje wersję projektu i decyzję
  człowieka z teczki transakcji, która pozwoliła ją przekazać.
/ Rachunki do aktu: numer, posiadacz i cel, np. zapłata ceny albo spłata
  hipoteki sprzedającej w jej banku. Wpis mówi tylko, że biuro sprawdziło
  rachunek i że człowiek go zatwierdził (rozstrzygnięcie biura R4). Nie
  podaje, jak go sprawdzono.
/ Kredyt kupującego: tylko to, czego potrzebuje akt, np. bank, kwota
  kredytu z decyzji i warunki wypłaty, gdy kancelaria o nie prosi.

== Od kancelarii

/ Lista dokumentów: jakich dokumentów notariusz wymaga, od której strony,
  w jakiej postaci (oryginał, odpis, skan) i do kiedy.
/ Projekt aktu: kolejne wersje z numerem i datą.
/ Termin: propozycja dnia, godziny i miejsca oraz potwierdzenie terminu.
/ Koszty: informacja o taksie notarialnej, opłatach i podatkach, które
  pobierze notariusz. Biuro przekazuje ją stronom i jej nie ocenia.
/ Pierwokup: kiedy uprawniony dostał zawiadomienie i czy złożył oświadczenie
  o wykonaniu prawa albo czy termin upłynął bez oświadczenia.
/ Po akcie: to, co kancelaria zdecyduje przekazać, np. że akt został
  sporządzony i że wniosek do księgi wieczystej został złożony, z datą.

== Termin aktu

Termin ma jeden wpis, który zmienia stan kolejnymi wpisami:

/ Dane: dzień, godzina, miejsce i czynność.
/ Potwierdzenie biura: decyzja człowieka z teczki transakcji (profil
  Koordynatora transakcji). Koordynator wpisuje potwierdzenie ze wskazaniem
  tej decyzji.
/ Potwierdzenie kancelarii: wpis Sekretariatu w imieniu kancelarii.
/ Stan: proponowany, potwierdzony przez obie strony, przesunięty, odwołany
  albo odbył się.

Termin jest potwierdzony dopiero wtedy, gdy potwierdziły go obie strony
pokoju.

== Wiadomości

Pytania i odpowiedzi, które nie mieszczą się w polach wyżej, np. „Czy
notariusz przyjmie zaświadczenie spółdzielni wystawione 20 dni przed
aktem?”. Wiadomość wskazuje wpis, którego dotyczy. Pytanie strony o skutki
prawne albo o podatki Koordynator przekazuje notariuszowi w wiadomości
i nie odpowiada na nie sam (profil Koordynatora transakcji).

== Czego nie wpisuje się

/ Przeciwdziałanie praniu pieniędzy: żadne wyniki, oceny, podejrzenia ani
  znacznik wątpliwości Kontrolera, w żadną stronę (rozdział „Podstawa
  prawna”).
/ Dane jednej strony, których akt nie wymaga: np. budżet kupującego, jego
  dochody, cena minimalna sprzedającej albo historia negocjacji
  (rozstrzygnięcie biura R1).
/ Analizy biura: analiza stanu prawnego i notatki agentów zostają w teczce
  transakcji. Do pokoju trafiają tylko uwagi stron, na które zgodził się
  człowiek.
/ Treść teczki notariusza: kancelaria przekazuje tylko to, co sama wybierze
  (rozstrzygnięcie biura R5).
/ Dane innych transakcji: jeden pokój dotyczy jednej transakcji.

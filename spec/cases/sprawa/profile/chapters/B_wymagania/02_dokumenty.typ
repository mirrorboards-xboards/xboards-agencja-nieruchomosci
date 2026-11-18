= Dokumenty

== Lista dokumentów

Każda teczka ma listę dokumentów. Typ sprawy mówi, jakie dokumenty są
w nim potrzebne. Rdzeń zapisuje przy każdym dokumencie to samo:

/ Rodzaj: np. odpis księgi wieczystej, zaświadczenie spółdzielni, umowa
  przedwstępna albo protokół zdawczo-odbiorczy.
/ Uczestnik albo nieruchomość: kogo albo czego dokument dotyczy.
/ Wystawca i data wystawienia: np. „Spółdzielnia Mieszkaniowa Piast,
  3 marca 2026 r.”.
/ Ważność: do kiedy dokument jest ważny i skąd to wiadomo. Termin podaje
  wystawca, przepis albo instytucja, która dokumentu wymaga. Przykład:
  świadectwo charakterystyki energetycznej jest ważne 10 lat od
  sporządzenia#footnote[Art. 14 ust. 1 uceb.], a zaświadczenie spółdzielni
  tak długo, jak przyjmuje je notariusz.
/ Kto wymaga: przepis, notariusz, bank albo umowa. Koordynator transakcji
  nie dokłada do listy własnych wymagań.
/ Pochodzenie: od kogo i którym kanałem dokument przyszedł, np. „skan od
  klientki pocztą elektroniczną” albo „odpis pobrany przez Koordynatora
  z centralnej bazy ksiąg wieczystych”. Oryginał, kopia i skan to różne
  rzeczy i rdzeń je rozróżnia.
/ Wniosek: gdy biuro występuje o dokument w imieniu strony, wpis wskazuje
  pełnomocnictwo, na podstawie którego to zrobiło, i datę wniosku.
/ Stan: wymagany, zamówiony, otrzymany, sprawdzony, odrzucony albo zbędny.
  Odrzucenie ma powód, np. „nieczytelna pieczęć”.
/ Plik: ścieżka w repozytorium plików teczki i skrót treści. Pliku się nie
  zmienia, także skanu dokumentu tożsamości.

Program liczy, czy dokument będzie ważny w dniu, w którym jest potrzebny,
np. w dniu aktu. Gdy nie będzie, agent sprawy zamawia nowy wcześniej.

== Pełnomocnictwa

Agent występuje w imieniu klienta tylko w granicach pełnomocnictwa, którego
klient udzielił biuru (dokument „Agencja”, zasada „W imieniu klienta tylko
z pełnomocnictwem”). Dlatego rdzeń zapisuje każde pełnomocnictwo osobno:

/ Mocodawca i pełnomocnik: uczestnik, który udziela pełnomocnictwa, i ten,
  kto je dostaje: biuro albo osoba.
/ Zakres: czynności, do których umocowuje, np. „wystąpienie do spółdzielni
  o zaświadczenie”, „uzgadnianie terminu aktu z kancelarią” albo „odbiór
  odpisu księgi”. Pełnomocnictwo ogólne obejmuje tylko czynności zwykłego
  zarządu. Do innych czynności pełnomocnictwo musi określać ich rodzaj.
  Gdy ustawa wymaga pełnomocnictwa do poszczególnej czynności, pełnomocnictwo
  wskazuje tę czynność#footnote[Art. 98 KC.].
/ Forma: pisemna, z podpisem notarialnie poświadczonym albo akt notarialny.
  Gdy czynność wymaga szczególnej formy, pełnomocnictwo ma tę samą
  formę#footnote[Art. 99 § 1 KC.]. Przykład: pełnomocnictwo do aktu
  sprzedaży ma formę aktu notarialnego.
/ Daty: udzielenia, a gdy pełnomocnictwo ma termin, także końca. Daty
  udzielenia się nie zmienia, bo na niej stoją wcześniejsze pisma.
/ Wygaśnięcie: pełnomocnictwo można odwołać w każdym czasie, chyba że
  mocodawca zrzekł się odwołania z przyczyn uzasadnionych stosunkiem
  prawnym. Umocowanie wygasa też ze śmiercią mocodawcy albo pełnomocnika.
  Umocowanie trwa mimo śmierci, gdy pełnomocnictwo tak zastrzega z przyczyn
  uzasadnionych stosunkiem prawnym, na którym stoi#footnote[Art. 101 § 1–2 KC.].
  Rdzeń zapisuje datę i źródło wiadomości o odwołaniu albo wygaśnięciu.
  Wygasłe pełnomocnictwo zostaje wygasłe. Ponowne umocowanie jest nowym
  wpisem.
/ Dokument: wiersz z listy dokumentów, na którym pełnomocnictwo stoi.

Zanim agent wystąpi w imieniu klienta, program porównuje czynność z zakresem
i terminem pełnomocnictwa. Czynność poza zakresem wymaga nowego
pełnomocnictwa, a nie zgody człowieka. Pismo w imieniu klienta wychodzi tylko
na pełnomocnictwie, które nie wygasło, zostało udzielone najpóźniej w dniu
wysłania pisma i jest w tym dniu ważne.

== Protokoły

Gdy nieruchomość albo klucze przechodzą z rąk do rąk, strony podpisują
protokół. Rdzeń zapisuje protokół w teczce każdego typu tak samo:

/ Rodzaj: wydanie, zwrot, przekazanie, przejęcie, przyjęcie kluczy albo
  zwrot kluczy. Przykłady: wydanie lokalu najemcy, przekazanie mieszkania
  kupującemu, przejęcie lokalu krótkoterminowego od właściciela i przyjęcie
  kluczy do prezentacji oferty.
/ Nieruchomość i chwila: czego protokół dotyczy i kiedy go spisano.
/ Pozycje: liczniki z numerem i stanem, wyposażenie, klucze, pomieszczenia
  i uwagi, np. wady. Pozycja może mieć zdjęcie ze ścieżką i skrótem pliku.
/ Stan: projekt albo podpisany. Podpisany protokół wskazuje dokument z listy
  dokumentów.

Po podpisie pozycji protokołu się nie zmienia. Program porównuje protokół
zwrotu z protokołem wydania, np. stan liczników i wyposażenia.

== Sprawdzenia

Kontroler klienta wpisuje wynik każdego sprawdzenia, które nie należy do
przeciwdziałania praniu pieniędzy. Typ sprawy mówi, co Kontroler w nim
sprawdza. Rdzeń zapisuje przy każdym wyniku to samo:

/ Rodzaj: tożsamość, tytuł do nieruchomości, pełnomocnictwo albo rachunek.
/ Sprawdzany wiersz: uczestnik, dokument, pełnomocnictwo albo rachunek
  z teczki. Wynik dotyczy tej wersji wiersza, którą Kontroler sprawdził.
  Gdy wiersz się zmieni, np. przyjdzie nowy numer rachunku, wynik go nie
  obejmuje.
/ Źródło spoza teczki: np. dział II księgi wieczystej z dnia sprawdzenia,
  odpis z KRS albo potwierdzenie posiadacza rachunku innym kanałem.
/ Wynik: zgodne, uwagi albo nie można ustalić. Uwagi i „nie można ustalić”
  mają opis, który mówi, czego brakuje.
/ Data i Kontroler: kiedy i który Kontroler sprawdził. Wynik ma jego podpis.

Ponowne sprawdzenie jest nowym wpisem, a poprzedni wynik staje się
nieaktualny. Wynik sprawdzenia rachunku, tytułu i pełnomocnictwa trafia do
teczki. Ustalenia z przeciwdziałania praniu pieniędzy do niej nie trafiają
(rozdział „Co rdzeń trzyma”).

*Znacznik wątpliwości.* Kontroler stawia go w metryce teczki bez powodu.
Zdejmuje go tylko decyzja człowieka podpisana jego kluczem. Dopóki znacznik
stoi, agent sprawy nie przygotowuje umowy z klientem do podpisu,
Koordynator transakcji nie potwierdza terminu aktu, a człowiek nie daje
zgody na rachunek. Dokumenty agent zbiera dalej. Kierownik działu widzi, że
znacznik stoi, ale nie pyta o powód.

= Katalogi danych

Każda pozycja ma w swoim folderze plik `board.toml`. Klucz `profile`
wskazuje jej profil, a lista `meta` rodzaje danych jej pokoju w SQL (XMeta).
Rodzaje własne leżą w folderze `meta/` pozycji i wpis `./meta/*` bierze je
wszystkie. Pozostałe wpisy importują rodzaje z `meta/` innych pozycji, np.
typ sprawy bierze cały rdzeń wpisem `../sprawa/meta/*`. Importy nie są
przechodnie. Dlatego kierownik działu importuje pamięć stanowiska i pamięć
kierownika działu obok siebie:

#table(
  columns: (auto, 1fr, 1fr),
  table.header[Pozycja][Katalog][Pokój],
  [`cases/sprawa`], [rdzeń wspólny teczek], [nie ma własnego pokoju; wchodzi do każdej teczki],
  [`cases/<typ>`], [typ sprawy; importuje rdzeń], [teczka każdej sprawy tego typu],
  [`cases/wymiana`], [pokój wymiany, wspólny dla biura i XNotariusza], [pokój wspólny z kancelarią notarialną na XNotariuszu (rozstrzygnięcie biura R5)],
  [`registers/<rejestr>`], [rejestr], [pokój stały, np. rejestr kontroli],
  [`positions/stanowisko`], [pamięć każdego stanowiska: ustalenia, źródła, polecenia i kontakty], [nie ma własnego pokoju; wchodzi do pokoju pamięci każdego agenta],
  [`positions/kierownik_dzialu`], [pamięć kierowników działów: agenci i ich wpuszczenia, raporty i prośby o decyzje, zajrzenia do teczek, standardy działu, zwyczaje kontrahentów, spis spraw dla Recepcji i rozesłanie polisy], [nie ma własnego pokoju; wchodzi do pokoju pamięci Kierownika pośrednictwa, Kierownika transakcji, Kierownika najmu i Kierownika kontroli],
  [`positions/<dział>/<stanowisko>`], [pamięć stanowiska; importuje potrzebne rodzaje pamięci każdego stanowiska, a u kierownika działu także pamięci kierownika działu], [pokój pamięci każdego agenta na tym stanowisku],
)

Działy to podgrupy `positions/`: `biuro`, `posrednictwo`, `transakcje`,
`najem` i `kontrola`.

Pamięć stanowiska importuje rodzaje pamięci wspólnej pojedynczo, np.
`../../stanowisko/meta/stanowisko-ustalenia`:

/ Każdy agent: `stanowisko-ustalenia`, `stanowisko-zrodla` i `stanowisko-polecenia`.
/ Kontakty: `stanowisko-kontakty` importuje tylko stanowisko, którego profil każe
  pamiętać kontakty.
/ Kierownik działu: rodzaje `positions/kierownik_dzialu`, które prowadzi.
  Kierownik kontroli nie prowadzi spisu spraw dla Recepcji
  (`kierownik-dzialu-spis`) ani rozesłania polisy (`kierownik-dzialu-polisy`).
  Kierownik transakcji nie prowadzi rozesłania polisy.
/ Bez rodzajów własnych: stanowisko, któremu wystarcza pamięć wspólna, nie ma
  folderu `meta/`; jego `board.toml` ma same importy.

Rodzaj, który wskazuje tabele innego rodzaju, wymaga tego rodzaju w pokoju.
Gdy go brakuje, `xmeta` kończy się błędem. Katalogi pamięci wspólnej nie
wskazują rodzajów stanowisk.

Do pokoju wymiany piszą biuro i kancelaria, więc obie aplikacje znają ten
sam katalog. Katalog leży w tym xboardzie, w `cases/wymiana`. Kancelaria
notarialna katalogu nie zmienia. Zmianę katalogu biuro uzgadnia
z XNotariuszem.

Hooki React generuje `xmeta codegen`, uruchomiony w korzeniu xboardu. Moduł
nazywa się ścieżką pozycji:
`@xboards-agencja-nieruchomosci/react/<ścieżka pozycji>/<rodzaj>`, np.
`…/cases/oferta/oferta-ogloszenia` albo `…/cases/sprawa/sprawa-metryka`.

Zasady wiążą każdy katalog:

/ Nazwy: rodzaj nazywa się `<dziedzina>-<rzecz>`. Dziedziną jest nazwa
  pozycji z łącznikami zamiast podkreśleń, np. `oferta-ogloszenia`.
  W rdzeniu dziedziną jest `sprawa`, np. `sprawa-uczestnicy`, w pamięci
  każdego stanowiska `stanowisko`, np. `stanowisko-ustalenia`, a w pamięci
  kierownika działu `kierownik-dzialu`, np. `kierownik-dzialu-raporty`.
  Tabele rodzaju zaczynają się od jego nazwy z podkreśleniami. Identyfikatory są po
  polsku, bez polskich znaków.
/ Typy danych: tabele są STRICT. Wartości wyliczeniowe mają
  `CHECK (… IN (…))`, a tak i nie to `INTEGER CHECK (… IN (0, 1))`. Daty
  i czasy są tekstem ISO 8601. Kwoty są liczbami całkowitymi w groszach.
  Każda reguła ma nazwany `CONSTRAINT` albo komunikat `RAISE` po polsku.
/ Identyfikatory: wiersz, na który wskazuje inny pokój albo który operacja
  musi od razu powiązać, ma identyfikator UUID nadany przez aplikację.
  Generator nie zna `RETURNING`.
/ Odwołania: klucz obcy łączy tylko wiersze jednego katalogu. Rodzaj może
  wskazywać też wiersze pozycji ogólnej, którą katalog importuje, np. typ
  sprawy wiersze rdzenia. Wiersz innego pokoju wskazuje się
  tekstem: identyfikatorem pokoju i wiersza, bez klucza obcego.
/ Tożsamość: ludzi i agentów wskazuje się identyfikatorem tożsamości
  z XAuth/xkeys. Baza nie sprawdza jego formatu. Stanowiska nazywa się tak
  jak ich foldery pod `positions/`.
/ Autor wpisu: autora każdego zapisu poświadcza serwer pokoju na podstawie
  podpisu żądania. Dziennik pokoju zapisuje autora przy każdej zmianie
  i tych zapisów się nie zmienia. Tabele nie mają więc kolumn autora.
  Wyjątkiem są decyzje i kolumny „kto”, których wymaga profil, np. kto
  potwierdził rachunek.
/ Decyzje i zgody: decyzja człowieka ma podpis kluczem z xkeys
  i tożsamość podpisującego. Podpis obejmuje skrót wskazanego wiersza
  i pliku, który liczy aplikacja. Baza wymaga obecności podpisu, a weryfikuje
  go aplikacja. Zgoda klienta nie ma klucza. Wskazuje dokument albo
  wiadomość, w której klient ją złożył.
/ Pliki: plik leży w repozytorium XGit pokoju. Tabela trzyma ścieżkę w tym
  repozytorium i skrót treści pliku. Pliku dokumentu się nie zmienia, także
  skanu dokumentu tożsamości.
/ Źródła: każda dana, którą biuro przekazuje na zewnątrz, ma wpis
  w `sprawa_dokumenty_zrodla`. Wpis wskazuje dokument albo osobę, od której
  dana pochodzi.
/ Przeciwdziałanie praniu pieniędzy i rozdział stron: wyniki środków
  bezpieczeństwa finansowego, ocena ryzyka i powód wątpliwości są tylko
  w rejestrze kontroli#footnote[Art. 54 ust. 1 pkt 2 AML.]. Teczka ma
  znacznik wątpliwości bez powodu. Teczka trzyma dane jednej strony albo
  dane wspólne obu stron (rozstrzygnięcie biura R1).
/ Typy na rdzeniu: typ sprawy nie zmienia tabel rdzenia. Może dodać na nich
  tylko triggery, które odmawiają zapisu (`RAISE(ABORT, …)`). Każdy typ ma
  rodzaj `<typ>-reguly`. Trzyma on etapy typu z dozwolonymi przejściami,
  role uczestników i rodzaje zgód. Rdzeń zna tylko etapy `przyjeta`,
  `przerwana` i `zamknieta`. Z etapu `przerwana` sprawa przechodzi do
  `zamknieta` albo wraca do etapu sprzed przerwania za decyzją
  `wznowienie_sprawy`. Typ dopuszcza ten powrót w swoich przejściach,
  a warunki wejścia w etap działają dalej.
/ Widoki rdzenia: trzy pojęcia, których używa każdy typ, rdzeń liczy
  w widokach. Typ nie powtarza ich zapytań. Zatwierdzona decyzja to wiersz
  widoku `sprawa_decyzje_zatwierdzone`. Rachunek dopuszczony to wiersz widoku
  `sprawa_rachunki_dopuszczone`. Dzień, od którego biuro może pracować dla
  konsumenta, podaje widok `sprawa_umowy_praca_od`.
/ Przechowywanie: wierszy się nie usuwa (rozstrzygnięcie biura R9).
  Pomyłkę poprawia się stanem, np. „zbędny” albo „anulowana”, albo nowym
  wierszem. Wyjątkiem są adresy i obywatelstwa uczestników oraz działki
  nieruchomości. Pomyłkę w nich poprawia się usunięciem wiersza, bo dziennik
  pokoju zachowuje poprzedni stan z autorem.
/ Bez zastępowania: każda tabela odmawia wpisu, który zastąpiłby istniejący
  wiersz. Tak działają `INSERT OR REPLACE` i upsert (`ON CONFLICT … DO
  UPDATE`). Te instrukcje omijają reguły zmiany i usuwania, dlatego tabela
  ich nie przyjmuje. Wyjątkiem są tabele poprawek, czyli adresy,
  obywatelstwa i działki. Operacje używają tylko zwykłych `INSERT`
  i `UPDATE`.
/ Droga zapisu: do pokoju z katalogiem pisze się przez SQLite autora.
  Autor wykonuje zapis u siebie, razem z regułami katalogu, a pokój dostaje
  gotowe strony bazy. Synchronizacja Turso odtwarza zmiany wierszy na
  drugiej bazie. Uruchamia wtedy reguły katalogu drugi raz, a samych reguł
  nie przenosi. Dlatego synchronizacja Turso do takich pokoi nie pisze.
/ Operacje: każda zaczyna się linią `-- name: <nazwa> :many|:one|:exec`
  i opisem po polsku. Odczyt nazywa się rzeczownikiem (`terminy`), a zapis
  czasownikiem (`wpisz_termin`). Kilka instrukcji jednej operacji `:exec`
  to jedna transakcja.
/ Migracje: `migrations/0001.sql` i kolejne. Migracji wydanej, czyli
  zapisanej w blokadzie, nie wolno zmienić. Zmiana to nowa migracja.
  Rodzaj nie wskazuje rodzaju z późniejszej migracji. Regułę, która zależy
  od późniejszego rodzaju, zapisuje migracja tego rodzaju.
/ Sprawdzenie: `xmeta codegen --check` i `tsc` w kliencie. Do tego
  scenariusze na SQLite (`python3 tests/run.py`):
  przebieg z profilu pozycji i złe stany, które baza ma odrzucić.
  Scenariusze próbują też zastąpić istniejący wiersz każdej tabeli przez
  `INSERT OR REPLACE` i upsert.

== Czego baza nie pilnuje

Części zasad z profili baza nie wymusza. Pilnuje ich aplikacja, program
albo uprawnienia pokoju. Trzy z nich dotyczą każdego katalogu:

/ Kto pisze: które stanowisko pisze do której tabeli i kto rozstrzyga
  decyzję. Pilnują tego uprawnienia pokoju, np. że opiekunowie stron piszą
  w teczce transakcji tylko notatki.
/ Odnośniki do innych pokoi: baza nie sprawdza, że odnośnik wskazuje
  istniejący wiersz właściwej teczki. Sprawdza to aplikacja.
/ Terminy: terminy z ustaw, umów i profili liczy program, np. 2 dni robocze
  na zawiadomienie, miesiąc na zwrot kaucji i przypomnienie 30 dni przed
  końcem polisy wykonawcy. Baza zapisuje tylko daty.

=== Rdzeń

/ Źródło danej: baza nie wymaga wpisu w `sprawa_dokumenty_zrodla` przy
  danej, którą biuro przekazuje na zewnątrz. Wymaga go aplikacja przed
  wysyłką.
/ Zgoda z warunkiem: baza liczy ją jak zgodę. Spełnienie warunku sprawdza
  aplikacja.
/ Wysyłka zawsze za zgodą: baza odróżnia taką wysyłkę tylko wtedy, gdy
  wskazuje ją wiersz typu sprawy. Projekt umowy o zarządzanie, aneks albo
  uwagi do projektu aktu spoza wzorów aplikacja kieruje do zgody sama.
/ Rozdział stron (R1): baza nie zna treści wysyłki. Że granice negocjacji,
  budżet i finanse klienta nie trafiają do drugiej strony, sprawdza
  aplikacja.

=== Oferta

/ Treść ogłoszenia: adres bez numeru lokalu, zdjęcia bez osób i dokumentów
  oraz dane zgodne z dokumentami. Sprawdza to aplikacja przed publikacją.
/ Ogłoszenie po zmianie ceny: baza sprawdza cenę przy publikacji i przy
  nowej wersji ogłoszenia. Nowej wersji po zmianie ceny wymaga aplikacja.
/ Zakres umowy: baza sprawdza w zakresie umowy pośrednictwa tylko publikację
  ogłoszeń. Prezentacje i wysyłanie ofert, także w poszukiwaniu, sprawdza
  aplikacja.
/ Pełny adres: dostaje go tylko osoba umówiona na prezentację. Pilnuje tego
  aplikacja.
/ Zlecający spoza księgi: gdy osoba, która daje ofertę, nie jest wpisana
  w księdze wieczystej, agent zgłasza to przed prezentacją. Pilnuje tego
  aplikacja.

=== Poszukiwanie

/ Oferta niepubliczna: zgodność propozycji z granicami zgody właściciela
  i ze stanem oferty w katalogu ofert sprawdza aplikacja.
/ Wynik analizy kredytowej: liczy go program. Baza wymaga tylko pliku
  wyniku.
/ Bez polecania banku (R2): treść analizy sprawdza aplikacja.
/ Budżet: gdy kontroferta przekracza budżet klienta, aplikacja ostrzega
  o tym, zanim klient ją potwierdzi.

=== Transakcja

/ Dokumenty na dzień aktu: ważność dokumentów w dniu aktu sprawdza
  aplikacja.
/ Zgoda małżonka: gdy nieruchomość jest we wspólności majątkowej, zgody
  małżonka wymaga aplikacja.
/ Sprawdzenia przed aktem: sprawdzenia stron i tytułu zbywcy przez
  Kontrolera przed aktem wymaga aplikacja.
/ Cel rachunku: zgodność celu rachunku z rodzajem płatności, np. zadatku
  albo ceny, sprawdza aplikacja.
/ Cudzoziemiec i grunt rolny: zezwolenie dla cudzoziemca i status rolnika
  nabywcy sprawdza aplikacja.

=== Najem

/ Najem okazjonalny: że wynajmujący nie prowadzi działalności w zakresie
  wynajmu, sprawdza aplikacja.
/ Próg 10 000 euro: teczka najmu nie przelicza czynszu na euro. Przelicza
  program, a objęcie ustawą zapisuje rejestr kontroli.
/ Podwyżka czynszu: próg 3%, uzasadnienie na żądanie najemcy i podwyżki
  według umowy najmu okazjonalnego i instytucjonalnego liczy program.
/ Wypowiedzenie: przyczyny z ustawy i dłuższy okres wypowiedzenia z umowy
  sprawdza aplikacja.
/ Waloryzacja kaucji: baza sprawdza tylko, że zwrot nie jest niższy od
  pobranej kaucji. Kwotę liczy program.
/ Wstąpienie w najem: kto wstąpił w najem po śmierci najemcy, ustala
  człowiek. Baza tego nie sprawdza.
/ Rodzaj najmu: wybór rodzaju najmu przed projektem umowy sprawdza
  aplikacja.
/ Inne wypłaty: baza ogranicza tylko wypłaty z wpłaty czynszu i z rozliczenia
  kaucji. Wypłatę z inną podstawą sprawdza aplikacja.
/ Dane dla gminy: wzór danych o czynszach daje aplikacja.

=== Lokal krótkoterminowy

/ Godziny: baza porównuje tylko dni. Godziny zameldowania i wymeldowania
  oraz instrukcję wejścia nie wcześniej niż o 10 rano pilnuje aplikacja.
/ Zmiany rezerwacji: wpis w `pobyty_zmiany` przy każdej zmianie wymaga
  aplikacja.
/ Najkrótszy pobyt: widełki z umowy sprawdza aplikacja.
/ Synchronizacja kanałów: przekazanie rezerwacji i blokad do kanałów
  w ciągu godziny robi program.
/ Dane gości: wykonawca i właściciel lokalu nie dostają danych gości.
  Pilnuje tego aplikacja.
/ Wydanie kodu (R13): baza odmawia kodu poza pobytem i oknem zlecenia.
  Komu agent podaje kod, pilnuje aplikacja.
/ Zapasy: stan i progi zamówień liczy program.
/ Wskaźniki zestawienia: obłożenie, średnią cenę i średnią ocenę liczy
  program.

=== Rejestry

/ Wykonawca po końcu umowy albo polisy: zlecenie leży w teczce lokalu,
  więc rejestr wykonawców go nie zatrzyma. Pilnuje tego aplikacja.
/ Katalog ofert: wpis po decyzji przy rozbieżności z księgą i pokazywanie
  oferty tylko w granicach zgody sprawdza aplikacja.
/ Rejestr kontroli: akceptację PEP przed nawiązaniem stosunków i stan „do
  rozstrzygnięcia” przy samym zarządzaniu sprawdza aplikacja, bo umowa
  leży w teczce.

=== Pokój wymiany

/ Treść wiadomości: do pokoju nie trafia nic z przeciwdziałania praniu
  pieniędzy ani dane jednej strony, których akt nie wymaga. Pilnuje tego
  aplikacja.
/ Dziennik notariusza (R15): pismo przychodzące wpisuje do dziennika
  korespondencyjnego XNotariusz. Pokój tego nie wymaga.

=== Pamięć wspólna

/ Prośby do właściciela biura: tylko Kontrola przekazuje prośby wprost
  właścicielowi, a pozostałe działy Kierownikowi biura. Pamięć działu nie
  wie, czyj jest pokój, więc pilnuje tego aplikacja.
/ Powołanie Kontrolera: tylko Kierownik kontroli powołuje Kontrolera.
  Pilnuje tego aplikacja.
/ Bez kopii rejestrów: agent nie przepisuje do pamięci wpisów rejestru
  kontroli. Pilnuje tego aplikacja.

=== Stanowiska

/ Konsjerż bez kartoteki gości: baza odrzuca źródło o rodzaju `osoba`.
  Źródło innego rodzaju, które wskazuje pobyt gościa, odrzuca aplikacja.
/ Kierownik biura: nowych spraw nie przydziela działowi Transakcje, choć
  baza na to pozwala. Pilnuje tego aplikacja.

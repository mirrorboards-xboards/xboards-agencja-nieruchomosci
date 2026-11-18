= Pokoje i dostęp

Dane biura leżą w zaszyfrowanych pokojach. Każdy pokój ma własną bazę
i własny katalog rodzajów danych.

/ Pokój sprawy: teczka jednej sprawy, np. oferty albo transakcji.
/ Pokój stały: rejestr prowadzony przez całe życie biura, np. rejestr
  kontroli.
/ Pokój wymiany: pokój jednej transakcji albo jednego oświadczenia najemcy
  przy najmie okazjonalnym lub instytucjonalnym, wspólny z kancelarią
  notarialną na XNotariuszu (rozstrzygnięcie biura R5).
/ Pamięć agenta: własny pokój agenta, do którego należą on i jego właściciel.

Agent sprawy należy tylko do teczki swojej sprawy, do swojej pamięci i do
wzorów. Wyjątkiem są pokoje opisane niżej, w których pracuje kilku agentów.

#table(
  columns: (auto, 1fr, 1fr, auto),
  table.header[Pokój][Pisze][Czyta][Nie należy],
  [Teczka oferty], [Opiekun oferty; Kontroler klienta], [Kierownik pośrednictwa i Kierownik kontroli (wgląd)], [Opiekun poszukującego],
  [Teczka poszukiwania], [Opiekun poszukującego; Analityk kredytowy; Kontroler klienta], [Kierownik pośrednictwa, Kierownik transakcji i Kierownik kontroli (wgląd)], [Opiekun oferty],
  [Teczka transakcji], [Koordynator transakcji; Analityk stanu prawnego; Kontrolerzy klientów biura; opiekunowie stron (notatki)], [Kierownik transakcji i Kierownik kontroli (wgląd)], [Analityk kredytowy],
  [Teczka najmu], [Opiekun najmu; Kontrolerzy klientów biura; Koordynator transakcji przy najmie okazjonalnym albo instytucjonalnym; opiekunowie stron (notatki)], [Kierownik najmu i Kierownik kontroli (wgląd)], [Analityk kredytowy],
  [Teczka lokalu krótkoterminowego], [Zarządca obiektu; Konsjerż; Koordynator operacji obszaru; Kontroler właściciela lokalu], [Kierownik najmu i Kierownik kontroli (wgląd)], [Recepcja],
  [Pokój wymiany], [Koordynator transakcji w imieniu biura; Sekretariat kancelarii notarialnej], [Kierownik transakcji (wgląd i członkowie po stronie biura)], [pozostali agenci biura],
  [Kartoteka klientów], [Kierownik pośrednictwa i Kierownik najmu (dane klientów z umów)], [Kierownik biura (rozpoznanie powracającego klienta); Kierownik kontroli], [agenci spraw, Recepcja],
  [Katalog ofert], [Kierownik pośrednictwa (dane ofert z teczek ofert, status i widoczność)], [Opiekunowie poszukujących (dobór ofert biura); Recepcja (numer oferty jako znak rozpoznawczy); Kierownik biura], [Opiekunowie ofert, pozostali agenci],
  [Rejestr wykonawców], [Kierownik najmu (wykonawcy i cenniki zatwierdzone przez człowieka)], [Koordynatorzy operacji], [pozostali agenci],
  [Rejestr kontroli], [Kierownik kontroli; ludzie wpisują swoje rozstrzygnięcia podpisane kluczem], [właściciel biura; osoba, której biuro powierzyło obowiązki z ustawy o przeciwdziałaniu praniu pieniędzy], [agenci spraw, Recepcja, Kierownik biura],
)

*Dlaczego tak.*

- Opiekun oferty i Opiekun poszukującego reprezentują przeciwne strony, więc
  żaden nie należy do teczki drugiego (rozstrzygnięcie biura R1). Teczka
  transakcji trzyma tylko to, co dotyczy obu stron i notariusza. Opiekunowie
  ją czytają i zostawiają w niej notatki.
- Przed transakcją opiekunowie stron porozumiewają się tak jak z biurem
  z zewnątrz: przez skrzynkę biura i Recepcję.
- Przyjęta oferta sprzedaży trafia do działu Transakcji, a przyjęta oferta
  najmu do działu Najmu. Przy najmie teczki transakcji się nie otwiera.
  Wspólna jest teczka najmu, do której opiekunowie stron przenoszą w notatkach
  dane potrzebne do umowy.
- Analityk kredytowy pracuje w teczce poszukiwania, bo dochody i zobowiązania
  kupującego należą do jego strony. Do teczki transakcji nie należy. Opiekun
  poszukującego przenosi do niej tylko to, czego potrzebuje transakcja, np.
  bank i termin wypłaty kredytu.
- Kontroler jest jeden na klienta i powstaje przy pierwszej umowie z nim
  (rozstrzygnięcie biura R7). Zaczyna w teczce oferty, poszukiwania albo
  lokalu. Do teczki transakcji albo najmu wpuszcza go kierownik działu, do
  którego teczka należy. Przed wypłatą dla właściciela lokalu sprawdza jego
  rachunek (rozstrzygnięcie biura R4). Wyniki sprawdzeń z przeciwdziałania
  praniu pieniędzy trafiają do rejestru kontroli, a nie do teczki.
- Kartoteka klientów pozwala rozpoznać powracającego klienta bez czytania
  teczek. Dane klienta przenosi do niej kierownik działu, w którym klient
  zawarł umowę.
- Katalog ofert trzyma każdą ofertę sprzedaży i najmu długoterminowego
  z tym, co klient pozwolił pokazać. Opiekun poszukującego nie należy do
  teczek ofert (rozstrzygnięcie biura R1). Bez katalogu nie mógłby dobierać
  swojemu klientowi ofert biura. Dane właściciela, dokładny adres i granice
  negocjacji zostają w teczce oferty.
- Rejestr wykonawców trzyma wykonawców i cenniki, które zatwierdził człowiek.
  Dzięki temu wszyscy Koordynatorzy operacji zlecają pracę tym samym
  wykonawcom na tych samych warunkach.
- Kierownik biura nie ma wglądu w teczki. Przydziela sprawy i pilnuje
  obłożenia na podstawie raportów kierowników.

Właścicielem pokoi stałych (kartoteki klientów, katalogu ofert, rejestru
wykonawców i rejestru kontroli) jest właściciel biura. Członków dodaje on albo kierownik, który
pokój prowadzi.

W teczce sprawy uprawnienia wyglądają tak:

#table(
  columns: (auto, 1fr),
  table.header[Kto][Uprawnienia w teczce],
  [Człowiek przyjmujący sprawę], [właściciel pokoju, wszystkie uprawnienia],
  [Kierownik działu], [dodawanie i usuwanie członków, wymiana klucza, prawa do treści, które przekazuje agentom],
  [Agent sprawy], [tworzenie i zmiana treści, bez zarządzania członkami],
)

Agenta innego działu wpuszcza do teczki kierownik działu, do którego teczka
należy, na prośbę kierownika, który tego agenta powołał. Tak trafia np.
Analityk kredytowy do teczki poszukiwania.

Żaden agent sprawy nie ma prawa dodawania członków. Agenci spraw czytają
niezaufaną pocztę, więc spreparowana wiadomość nie może ich nakłonić do
wpuszczenia obcego klucza do teczki.

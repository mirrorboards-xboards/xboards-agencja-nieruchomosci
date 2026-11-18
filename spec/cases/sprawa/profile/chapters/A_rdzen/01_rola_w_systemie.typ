= Rola w systemie

Biuro prowadzi pięć typów teczek: ofertę, poszukiwanie, transakcję, najem
i lokal krótkoterminowy (dokument „Agencja nieruchomości”, rozdział „Pokoje i dostęp”). Każda
teczka jest osobnym zaszyfrowanym pokojem. Mimo różnych typów każda teczka
zapisuje część danych tak samo. W każdej są osoby z danymi, dokumenty,
decyzje człowieka, notatki przekazania, terminy i wysyłki. Tę wspólną część
nazywamy rdzeniem.

Rdzeń nie jest osobnym pokojem. Jego dane powstają w teczce każdego typu,
obok danych tego typu. Typ sprawy opisuje tylko swoje dane, np. ogłoszenia
oferty albo kaucję najmu, i wskazuje wiersze rdzenia. Przykład: kaucja
w teczce najmu wskazuje najemcę z listy uczestników rdzenia i rachunek
z listy rachunków rdzenia.

Dzięki rdzeniowi każde stanowisko pracuje tak samo w każdej teczce:

/ Agent sprawy: prowadzi uczestników, dokumenty, terminy, wysyłki i notatki
  swojej teczki, np. Opiekun oferty, Koordynator transakcji albo Zarządca
  obiektu.
/ Kontroler klienta: wpisuje wyniki sprawdzeń tożsamości, tytułu,
  pełnomocnictw i rachunków oraz znacznik wątpliwości.
/ Człowiek: rozstrzyga prośby o decyzje i podpisuje rozstrzygnięcie
  własnym kluczem.
/ Kierownik działu: zarządza członkami teczki i zagląda do niej tylko
  z powodem. Treści teczki nie zmienia (dokument „Agencja nieruchomości”, rozdział
  „Kierownik działu”).

Kiedy człowiek albo agent zajmuje stanowisko, pracuje na tych samych danych.
Sprawę można więc przekazać między człowiekiem a agentem bez zmiany
przebiegu (dokument „Agencja nieruchomości”, zasada „Człowiek albo agent”).

Z tego profilu powstał katalog danych rdzenia w `cases/sprawa/meta`.
Katalog każdego typu sprawy go importuje. Zmiana rdzenia dotyczy więc teczek
wszystkich typów naraz.

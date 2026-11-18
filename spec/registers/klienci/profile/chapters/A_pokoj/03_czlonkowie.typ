= Członkowie

#table(
  columns: (auto, 1fr),
  table.header[Kto][Rola w pokoju],
  [Właściciel biura], [Właściciel pokoju, wszystkie uprawnienia. Dodaje i usuwa członków, a przy usunięciu wymienia klucz pokoju.],
  [Kierownik pośrednictwa], [Pisze i czyta. Wpisuje klientów z umów pośrednictwa i aktualizuje ich dane. Dane powracającego klienta przekazuje nowemu agentowi na start.],
  [Kierownik najmu], [Pisze i czyta. Wpisuje klientów z umów o zarządzanie nieruchomością i aktualizuje ich dane. Dane powracającego klienta przekazuje nowemu agentowi na start.],
  [Kierownik biura], [Czyta. Przy każdej nowej karcie Recepcji sprawdza, czy zgłaszający był już klientem biura. Nie pisze.],
  [Kierownik kontroli], [Czyta. Sprawdza, czy klient był już klientem biura i ma Kontrolera, i znajduje umowy klienta, gdy przygotowuje odpowiedź na prośbę o dane. Nie pisze.],
)

Członków dodaje właściciel biura albo kierownik, który pokój prowadzi
(dokument „Agencja nieruchomości”, rozdział „Pokoje i dostęp”). Kierownik dodaje tylko
stanowiska z tej tabeli, np. następcę Kierownika biura po jego wymianie.
Kogokolwiek innego dodaje tylko za zgodą właściciela biura.

*Kto nie należy.* Agenci spraw, Recepcja, Kontrolerzy, Kierownik transakcji,
Koordynatorzy operacji i ludzie spoza biura nie należą do pokoju.

/ Agenci spraw: agent jednej sprawy nie widzi danych innych klientów
  (dokument „Agencja nieruchomości”, zasada „Tylko potrzebny dostęp”). Dane powracającego
  klienta dostaje od kierownika działu.
/ Recepcja: czyta niezaufaną pocztę od wszystkich. Powracającego klienta
  rozpoznaje Kierownik biura, a nie ona.
/ Kierownik transakcji: transakcja nie zaczyna się od umowy z klientem, tylko
  od oferty przyjętej w Pośrednictwie. Klienci transakcji są już
  w kartotece.
/ Kontrolerzy: Kontroler zna swojego klienta z teczek i z danych, które daje
  mu na start Kierownik kontroli.

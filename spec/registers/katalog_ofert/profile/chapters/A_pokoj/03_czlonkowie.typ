= Członkowie

#table(
  columns: (auto, 1fr),
  table.header[Kto][Rola w pokoju],
  [Właściciel biura], [Właściciel pokoju, wszystkie uprawnienia. Dodaje i usuwa członków, a przy usunięciu wymienia klucz pokoju.],
  [Kierownik pośrednictwa], [Pisze i czyta. Przenosi dane ofert z teczek ofert, zmienia status i widoczność ofert. Dodaje do pokoju Opiekunów poszukujących.],
  [Opiekunowie poszukujących], [Czytają. Dobierają oferty biura dla swoich klientów. Ofertę niepubliczną pokazują klientowi tylko w granicach zgody właściciela nieruchomości. Nie piszą.],
  [Recepcja], [Czyta. Szuka w katalogu numeru oferty albo ogłoszenia z wiadomości. Wpis wskazuje teczkę oferty, a Recepcja oddaje wiadomość jej Opiekunowi oferty. Nie pisze.],
  [Kierownik biura], [Czyta. Sprawdza ofertę, gdy karta Recepcji ją wskazuje, i widzi, jakie oferty prowadzi biuro. Nie pisze.],
)

Członków dodaje właściciel biura albo Kierownik pośrednictwa (dokument
„Agencja nieruchomości”, rozdział „Pokoje i dostęp”). Kierownik pośrednictwa dodaje Opiekuna
poszukującego w dniu jego powołania. Usuwa go razem z wymianą klucza pokoju,
gdy Opiekun zaśnie po raporcie końcowym albo zostanie wymieniony.
Kogokolwiek innego dodaje tylko za zgodą właściciela biura.

*Kto nie należy.*

/ Opiekunowie ofert: dane swojej oferty mają w teczce oferty. Zmiany do
  katalogu zgłaszają Kierownikowi pośrednictwa w raporcie. Cudzych ofert do
  pracy nie potrzebują (dokument „Agencja nieruchomości”, zasada „Tylko potrzebny dostęp”).
/ Kontrolerzy, Analitycy, Kierownik kontroli oraz działy Transakcji i Najmu:
  nie dobierają ofert i nie potrzebują katalogu do pracy.
/ Zarządcy obiektów: lokale krótkoterminowe nie należą do katalogu.
/ Klienci, portale i inne biura: to strony spoza biura. Ogłoszenia na
  portale publikuje Opiekun oferty z teczki oferty, a nie z katalogu.

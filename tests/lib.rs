//! The scenarios of the agency's catalogs. Each module checks one room on SQLite in memory: the
//! flow of the profile of its position and the wrong states its database refuses. The room and
//! its operations are the generated Rust client, so a scenario calls the operations the app calls.
//!
//! Some rules of the tables guard the data against writers outside the client: another app,
//! hand-written SQL or a sync. Such a writer can leave out a value an operation requires or give
//! a value outside a list. The typed operations cannot make that write, so a scenario writes it
//! itself with `MemoryRoom::execute` and expects the table to refuse it.

#[cfg(test)]
mod analityk_stanu_prawnego;
#[cfg(test)]
mod expect;
#[cfg(test)]
mod katalog_ofert;
#[cfg(test)]
mod kierownicy_bez_rodzajow;
#[cfg(test)]
mod kierownik_biura;
#[cfg(test)]
mod kierownik_dzialu;
#[cfg(test)]
mod kierownik_kontroli;
#[cfg(test)]
mod kierownik_najmu;
#[cfg(test)]
mod kierownik_posrednictwa;
#[cfg(test)]
mod klienci;
#[cfg(test)]
mod konsjerz;
#[cfg(test)]
mod kontroler;
#[cfg(test)]
mod oferta;
#[cfg(test)]
mod poszukiwanie;
#[cfg(test)]
mod recepcja;
#[cfg(test)]
mod rejestr_kontroli;
#[cfg(test)]
mod sprawa;
#[cfg(test)]
mod sprawa_dodatki;
#[cfg(test)]
mod sprawa_porzadek;
#[cfg(test)]
mod stanowiska_bez_rodzajow;
#[cfg(test)]
mod stanowisko;
#[cfg(test)]
mod transakcja;
#[cfg(test)]
mod wykonawcy;
#[cfg(test)]
mod wymiana;

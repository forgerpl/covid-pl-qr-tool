use chrono::naive::NaiveDate;
use std::convert::Infallible;
use std::str;
use std::str::FromStr;

use crate::error::{FieldName, MalformedLine};

#[derive(Debug, Clone, PartialEq)]
pub struct VaccinationInfo {
    // szczepienieId – identyfikator szczepienia (identyfikator zasobu Immunization),
    id: u64,
    // wersjaZasobu – wersja zasobu szczepienia (Immunization),
    version: u8,
    // dataWydania - data wydania dowodu szczepienia,
    issue_date: NaiveDate,
    // imiona – imiona posiadacza dowodu szczepienia,
    names: String,
    // pierwszaLiteraNazwiska – pierwsza litera nazwiska posiadacza dowodu szczepienia,
    first_surname_letter: char,
    // skroconaDataUrodzenia – skrócona data urodzenia posiadacza dowodu szczepienia,
    short_birthdate: NaiveDate,
    // dataWaznosciDowodu – data ważności dowodu,
    certificate_expiration: NaiveDate,
    // danaTechniczna – oznaczenie szczepionki
    vaccine_type: String,
}

#[inline]
fn munch<'a, T, E>(
    field: FieldName,
    mut split: impl Iterator<Item = &'a str>,
    parser: impl Fn(&'a str) -> Result<T, E>,
) -> Result<T, MalformedLine> {
    if let Some(value) = split.next() {
        parser(value).map_err(|_| MalformedLine::MalformedFieldData(field))
    } else {
        Err(MalformedLine::MissingField(field))
    }
}

impl FromStr for VaccinationInfo {
    type Err = MalformedLine;

    fn from_str(line: &str) -> Result<Self, Self::Err> {
        static DATE_FORMAT: &str = "%d-%m-%Y";

        let mut s = line.split(';');

        let id = munch(FieldName::Id, &mut s, |id| u64::from_str(id))?;
        let version = munch(FieldName::Version, &mut s, |v| {
            let version = u8::from_str(v).map_err(|_| ())?;

            // only version 1 is supported
            if version == 1 {
                Ok(version)
            } else {
                Err(())
            }
        })?;
        let issue_date = munch(FieldName::IssueDate, &mut s, |d| {
            NaiveDate::parse_from_str(d, DATE_FORMAT)
        })?;
        let names = munch(FieldName::Names, &mut s, |names| {
            Ok::<_, Infallible>(names.to_owned())
        })?;
        let first_surname_letter = munch(FieldName::FirstSurnameLetter, &mut s, |l| {
            l.chars().next().ok_or(())
        })?;
        let short_birthdate = munch(FieldName::ShortBirthdate, &mut s, |d| {
            let source = format!("{}-0", d);

            NaiveDate::parse_from_str(&source, DATE_FORMAT)
        })?;
        let certificate_expiration = munch(FieldName::CertificateExpiration, &mut s, |d| {
            NaiveDate::parse_from_str(d, DATE_FORMAT)
        })?;
        let vaccine_type = munch(FieldName::VaccineType, &mut s, |vtype| {
            Ok::<_, Infallible>(vtype.to_owned())
        })?;

        let vi = VaccinationInfo {
            id,
            version,
            issue_date,
            names,
            first_surname_letter,
            short_birthdate,
            certificate_expiration,
            vaccine_type,
        };

        Ok(vi)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert2::check;

    mod parser {
        use super::*;

        #[test]
        fn parse() -> Result<(), MalformedLine> {
            static LINE: &str = "123456;1;20-01-2021;Anna Kowalska;M;17-04;20-01-2022;321";

            let parsed = VaccinationInfo::from_str(&LINE)?;

            check!(
                VaccinationInfo {
                    id: 123456,
                    version: 1,
                    issue_date: NaiveDate::from_ymd(2021, 01, 20),
                    names: "Anna Kowalska".to_owned(),
                    first_surname_letter: 'M',
                    short_birthdate: NaiveDate::from_ymd(0, 4, 17),
                    certificate_expiration: NaiveDate::from_ymd(2022, 01, 20),
                    vaccine_type: "321".to_owned(),
                } == parsed
            );

            Ok(())
        }

        mod malformed {
            use super::*;
            use assert2::let_assert;

            #[test]
            fn id() {
                let_assert!(Err(e) = VaccinationInfo::from_str("xxx"));
                check!(e == MalformedLine::MalformedFieldData(FieldName::Id));
            }

            #[test]
            fn version() {
                let_assert!(Err(e) = VaccinationInfo::from_str("123;xxx"));
                check!(e == MalformedLine::MalformedFieldData(FieldName::Version));
            }

            #[test]
            fn unknown_version() {
                let_assert!(Err(e) = VaccinationInfo::from_str("123;2"));
                check!(e == MalformedLine::MalformedFieldData(FieldName::Version));
            }

            #[test]
            fn issue_date() {
                let_assert!(Err(e) = VaccinationInfo::from_str("123;1;12-21-42"));
                check!(e == MalformedLine::MalformedFieldData(FieldName::IssueDate));
            }
        }
    }
}

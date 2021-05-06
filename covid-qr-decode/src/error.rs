use displaythis::Display;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DecodeError {
    /// Malformed info string
    #[error("malformed input line {0:?}")]
    MalformedLine(#[from] MalformedLine),
}

#[derive(Display, Debug, Copy, Clone, PartialEq)]
pub enum FieldName {
    #[display("szczepienieId")]
    Id,
    #[display("wersjaZasobu")]
    Version,
    #[display("dataWydania")]
    IssueDate,
    #[display("imiona")]
    Names,
    #[display("pierwszaLiteraNazwiska")]
    FirstSurnameLetter,
    #[display("skroconaDataUrodzenia")]
    ShortBirthdate,
    #[display("dataWaznosciDowodu")]
    CertificateExpiration,
    #[display("danaTechniczna")]
    VaccineType,
}

#[derive(Debug, Error, PartialEq)]
pub enum MalformedLine {
    #[error("missing input field: {0:?}")]
    MissingField(FieldName),
    #[error("malformed field data: {0:?}")]
    MalformedFieldData(FieldName),
}

#[derive(Debug, Error)]
pub enum DecryptionError {
    #[error("SSL decryption error: {0}")]
    Ssl(#[from] openssl::error::ErrorStack),
    #[error("Invalid UTF-8 string in the payload")]
    InvalidUtf8(#[from] std::str::Utf8Error),
    #[error("Empty payload; probably bad decryption key")]
    NoData,
}

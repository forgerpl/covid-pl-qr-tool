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

#[derive(Debug, Error)]
pub enum QrError {
    #[error("QR code either not found or not supported")]
    NoData,
    #[error("QR code extraction failed: {0}")]
    Extract(#[from] quircs::ExtractError),
    #[error("QR code decode failed: {0}")]
    Decode(#[from] quircs::DecodeError),
    #[error("QR image read failed: {0}")]
    Image(#[from] image::ImageError),
    #[error("Invalid UTF-8 string in the payload: {0}")]
    InvalidUtf8(#[from] std::str::Utf8Error),
    #[error("QR code payload version {0} not supported")]
    UnknownPayloadVersion(u8),
    #[error("QR code payload malformed")]
    MalformedPayload,
    #[error("QR code payload base64 error {0}")]
    MalformedPayloadBase64(#[from] base64::DecodeError),
}

#[derive(Debug, Error)]
pub enum PdfError {
    #[error("PDF processing error")]
    PdfProcessing(#[from] pdf::error::PdfError),
    #[error("QR code not found")]
    QrNotFound,
    #[error("Image conversion error")]
    ImageConversion,
}

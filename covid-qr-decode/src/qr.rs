use crate::error::QrError;
use image::DynamicImage;
use std::path::Path;

#[derive(Debug, Clone, Default)]
pub struct QrDecoder {
    decoder: quircs::Quirc,
}

impl QrDecoder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn read_image(&mut self, image_path: impl AsRef<Path>) -> Result<Vec<u8>, QrError> {
        let img = image::open(image_path)?;

        self.image_extract_encrypted(img)
    }

    pub fn image_extract_encrypted(&mut self, image: DynamicImage) -> Result<Vec<u8>, QrError> {
        let code = self.image_get_payload(image)?;
        Self::decode_payload(&code)
    }

    pub fn image_get_payload(&mut self, image: DynamicImage) -> Result<String, QrError> {
        // convert to gray scale
        let image = image.into_luma8();

        // identify all qr codes
        let mut codes =
            self.decoder
                .identify(image.width() as usize, image.height() as usize, &image);

        // only look up the first one
        let code = codes.next().ok_or(QrError::NoData)?;

        // see if it's properly extracted
        let code = code?;

        // decode the payload
        let data = code.decode()?;

        // convert to str from bytes
        let code = std::str::from_utf8(&data.payload)?;

        Ok(code.to_string())
    }

    pub fn decode_payload(code: &str) -> Result<Vec<u8>, QrError> {
        match code.split_once(';') {
            Some(("1", payload)) => {
                let decoded = base64::decode(payload)?;

                Ok(decoded)
            }
            Some((v, _)) => {
                if let Ok(v) = v.parse::<u8>() {
                    Err(QrError::UnknownPayloadVersion(v))
                } else {
                    Err(QrError::MalformedPayload)
                }
            }
            _ => Err(QrError::MalformedPayload),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert2::{check, let_assert};

    mod data {
        use super::*;

        pub fn case1() -> (impl AsRef<Path>, &'static str, &'static str, &'static [u8]) {
            let path = "testdata/1.png";
            let base64 = include_str!("../testdata/1.base64");
            let payload = include_str!("../testdata/1.payload");
            let cipher = include_bytes!("../testdata/1.cipher");

            (path, base64, payload, &cipher[..])
        }

        pub fn case2() -> (impl AsRef<Path>, &'static str, &'static str, &'static [u8]) {
            let path = "testdata/2.png";
            let base64 = include_str!("../testdata/2.base64");
            let payload = include_str!("../testdata/2.payload");
            let cipher = include_bytes!("../testdata/2.cipher");

            (path, base64, payload, &cipher[..])
        }
    }

    mod case1 {
        use super::*;

        #[test]
        fn full() {
            let (path, base64, payload, cipher) = data::case1();

            let mut qr = QrDecoder::new();

            let_assert!(Ok(decoded) = qr.read_image(path));
            check!(decoded == cipher);
        }

        #[test]
        fn payload_from_image() {
            let (path, base64, payload, cipher) = data::case1();

            let mut qr = QrDecoder::new();
            let img = image::open(path).unwrap();

            let_assert!(Ok(decoded) = qr.image_get_payload(img));
            check!(decoded == payload);
        }

        #[test]
        fn base64_from_image() {
            let (path, base64, payload, cipher) = data::case1();

            let mut qr = QrDecoder::new();
            let img = image::open(path).unwrap();

            let_assert!(Ok(decoded) = qr.image_get_payload(img));
            let decoded_b64 = decoded
                .strip_prefix("1;")
                .expect("payload should start with a version specifier");

            check!(decoded_b64 == base64);
        }

        #[test]
        fn cipher_from_image() {
            let (path, base64, payload, cipher) = data::case1();

            let mut qr = QrDecoder::new();
            let img = image::open(path).unwrap();

            let_assert!(Ok(decoded) = qr.image_extract_encrypted(img));
            check!(decoded == cipher);
        }
    }

    mod case2 {
        use super::*;

        #[test]
        fn full() {
            let (path, base64, payload, cipher) = data::case2();

            let mut qr = QrDecoder::new();

            let_assert!(Ok(decoded) = qr.read_image(path));
            check!(decoded == cipher);
        }
    }

    #[test]
    fn malformed_qr() {
        let mut qr = QrDecoder::new();

        let_assert!(Err(QrError::Decode(_)) = qr.read_image("testdata/malformed_qr.png"));
    }

    #[test]
    fn missing_qr() {
        let mut qr = QrDecoder::new();

        let_assert!(Err(QrError::NoData) = qr.read_image("testdata/missing_qr.png"));
    }
}

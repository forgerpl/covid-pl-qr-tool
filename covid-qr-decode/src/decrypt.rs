use crate::error::DecryptionError;
use openssl::{
    pkey::Public,
    rsa::{Padding, Rsa},
};

pub struct RsaDecrypter {
    key: Rsa<Public>,
}

impl RsaDecrypter {
    const DEFAULT_PADDING: Padding = Padding::PKCS1;

    pub fn new(key: Rsa<Public>) -> Self {
        RsaDecrypter { key }
    }

    pub fn decrypt(&self, ciphertext: impl AsRef<[u8]>) -> Result<String, DecryptionError> {
        let mut buf: Vec<u8> = vec![0; self.key.size() as usize];
        let ciphertext = ciphertext.as_ref();
        let out_len = self
            .key
            .public_decrypt(ciphertext, &mut buf, Self::DEFAULT_PADDING)?;

        if out_len == 0 {
            Err(DecryptionError::NoData)
        } else {
            buf.truncate(out_len);

            // try to decode as utf-8
            let s = std::str::from_utf8(&buf)?;

            Ok(s.to_string())
        }
    }
}

impl Default for RsaDecrypter {
    fn default() -> Self {
        let pem = include_bytes!("../keys/publiczny_klucz_podpisu.pub");
        let key = Rsa::public_key_from_pem(&pem[..]).expect("Malformed default RSA key");

        RsaDecrypter { key }
    }
}

impl From<Rsa<Public>> for RsaDecrypter {
    fn from(key: Rsa<Public>) -> Self {
        RsaDecrypter { key }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert2::{check, let_assert};

    mod data {
        use super::*;

        pub fn decrypter() -> RsaDecrypter {
            let pem = include_bytes!("../keys/test_public.pem");
            let key = Rsa::public_key_from_pem(&pem[..]).expect("Malformed test RSA key");

            RsaDecrypter::from(key)
        }

        pub fn case1() -> (&'static str, &'static [u8]) {
            let plain = include_str!("../testdata/1.plain");
            let cipher = include_bytes!("../testdata/1.cipher");

            (&plain[..], &cipher[..])
        }

        pub fn case2() -> (&'static str, &'static [u8]) {
            let plain = include_str!("../testdata/2.plain");
            let cipher = include_bytes!("../testdata/2.cipher");

            (&plain[..], &cipher[..])
        }

        pub fn malformed() -> &'static [u8] {
            let cipher = include_bytes!("../testdata/malformed.cipher");

            &cipher[..]
        }

        pub fn empty() -> &'static [u8] {
            let cipher = include_bytes!("../testdata/empty.cipher");

            &cipher[..]
        }

        pub fn invalid_utf8() -> &'static [u8] {
            let cipher = include_bytes!("../testdata/invalid_utf8.cipher");

            &cipher[..]
        }
    }

    #[test]
    fn correct1() {
        let (plain, cipher) = data::case1();

        let dec = data::decrypter();

        let_assert!(Ok(dec_plain) = dec.decrypt(&cipher));
        check!(dec_plain == plain);
    }

    #[test]
    fn correct2() {
        let (plain, cipher) = data::case2();

        let dec = data::decrypter();

        let_assert!(Ok(dec_plain) = dec.decrypt(&cipher));
        check!(dec_plain == plain);
    }

    #[test]
    fn empty() {
        let cipher = data::empty();

        let dec = data::decrypter();

        let_assert!(Err(DecryptionError::NoData) = dec.decrypt(&cipher));
    }

    #[test]
    fn malformed() {
        let cipher = data::malformed();

        let dec = data::decrypter();

        let_assert!(Err(DecryptionError::Ssl(_)) = dec.decrypt(&cipher));
    }

    #[test]
    fn invalid_utf8() {
        let cipher = data::invalid_utf8();

        let dec = data::decrypter();

        let_assert!(Err(DecryptionError::InvalidUtf8(_)) = dec.decrypt(&cipher));
    }
}

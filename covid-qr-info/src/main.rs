use covid_qr_decode::{PdfQrExtractor, QrDecoder, RsaDecrypter, VaccinationInfo};
use std::fs::{metadata, File};
use std::io::{self, BufReader, Read};
use std::path::Path;
use std::str::FromStr;

mod cli;

type Payload = Vec<u8>;

const ENCRYPTED_PAYLOAD_LEN: u64 = 256;

#[inline]
fn read_to_string(path: impl AsRef<Path>) -> io::Result<String> {
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);
    let mut data = String::new();

    reader.read_to_string(&mut data)?;

    Ok(data)
}

fn main() -> anyhow::Result<()> {
    let args = cli::get_matches();

    let (auto_pdf, auto_qr, auto_base64, auto_encrypted, auto_record) = match args.value_of("auto")
    {
        Some(path) => {
            let (mut auto_pdf, mut auto_qr, mut auto_base64, mut auto_encrypted, mut auto_record) =
                (None, None, None, None, None);

            match tree_magic_mini::from_filepath(path.as_ref()) {
                Some(pdf) if pdf == "application/pdf" => auto_pdf = Some(path),
                Some(image) if image.starts_with("image/") => auto_qr = Some(path),
                Some(text) if text == "text/plain" => {
                    // binary ciphertext will also be recognized as text/plain
                    // so try a file size heuristic
                    match metadata(path) {
                        Ok(meta) if meta.len() == ENCRYPTED_PAYLOAD_LEN => {
                            auto_encrypted = Some(path);
                        }
                        Ok(_) => {
                            // this can be base64 or record
                            // check if there's a separator present
                            // this will read the contents twice, but it's not a frequently used path anyway

                            if let Ok(data) = read_to_string(path) {
                                // base64 alphabet doesn't contain the separator
                                // but the qr code text payload contains ';', which separates version
                                // information and the base64-encoded payload itself

                                if let Some((_, b)) = data.split_once(';') {
                                    // at this point it's either 1;base64 or a record

                                    if b.contains(';') {
                                        auto_record = Some(path);
                                    } else {
                                        auto_base64 = Some(path);
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            // error at this point is fatal

                            anyhow::bail!("Failed to read plaintext file: {}", e);
                        }
                    }
                }
                Some(mime) => {
                    anyhow::bail!("Unable to process file type {}", mime);
                }
                None => {
                    anyhow::bail!("Unable to process unknonw file type");
                }
            }

            (auto_pdf, auto_qr, auto_base64, auto_encrypted, auto_record)
        }
        None => (None, None, None, None, None),
    };

    let pdf_payload = auto_pdf
        .or(args.value_of("pdf"))
        .map(|path| -> anyhow::Result<Payload> {
            let pdf = PdfQrExtractor::with_path(path)?;

            let mut qr = QrDecoder::new();

            let payload = pdf
                .images()
                .filter_map(|image| image.ok())
                .filter_map(move |image| qr.image_extract_encrypted(image).ok())
                .next()
                .ok_or_else(|| anyhow::anyhow!("Unable to find QR code in the PDF file"))?;

            Ok(payload)
        });

    let qr_payload = || {
        auto_qr
            .or(args.value_of("image"))
            .map(|path| -> anyhow::Result<Payload> {
                let mut qr = QrDecoder::new();

                Ok(qr.read_image(path).map_err(|e| {
                    anyhow::anyhow!("Unable to find QR code in the PDF file, {}", e)
                })?)
            })
    };

    let base64_payload = || {
        auto_base64
            .or(args.value_of("base64"))
            .map(|path| -> anyhow::Result<Payload> {
                let data = read_to_string(path)?;

                // decode
                let payload = QrDecoder::decode_payload(&data)?;

                Ok(payload)
            })
    };

    let ciphertext = || {
        auto_encrypted
            .or(args.value_of("encrypted"))
            .map(|path| -> anyhow::Result<Payload> {
                let file = File::open(path)?;
                let mut reader = BufReader::new(file);
                let mut data = Vec::new();

                reader.read_to_end(&mut data)?;

                Ok(data)
            })
    };

    let record = if let Some(payload) = pdf_payload
        .or_else(qr_payload)
        .or_else(base64_payload)
        .or_else(ciphertext)
    {
        // decrypt & verify
        let decrypter = RsaDecrypter::default();
        let record = decrypter
            .decrypt(payload?)
            .map_err(|_e| anyhow::anyhow!("Invalid cryptographic signature"))?;

        Ok(record)
    } else {
        let record = auto_record
            .or(args.value_of("record"))
            .map(|path| -> anyhow::Result<String> {
                let data = read_to_string(path)?;

                Ok(data)
            })
            .ok_or_else(|| anyhow::anyhow!("No payload found"))?;

        record
    };

    let record = record?;

    let record = VaccinationInfo::from_str(&record)?;

    println!(
        "{} vaccination certificate",
        if record.has_expired() {
            "Expired"
        } else {
            "Valid"
        }
    );
    println!("{:#?}", record);

    Ok(())
}

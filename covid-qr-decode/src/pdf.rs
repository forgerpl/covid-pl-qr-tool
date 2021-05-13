use crate::error::PdfError;
use image::DynamicImage;
use pdf::file::File;
use pdf::object::*;
use std::path::Path;
use std::rc::Rc;

#[derive(Clone)]
pub struct PdfQrExtractor {
    pdf: Rc<pdf::file::File<Vec<u8>>>,
}

impl PdfQrExtractor {
    pub fn with_path(path: impl AsRef<Path>) -> Result<Self, PdfError> {
        Ok(Self {
            pdf: Rc::new(File::open(path)?),
        })
    }

    pub fn images(&self) -> impl Iterator<Item = Result<DynamicImage, PdfError>> + '_ {
        self.pdf
            .pages()
            .flat_map(|page| page.ok())
            .flat_map(|page| page.resources().map(|res| res.clone()).ok())
            .flat_map({
                move |resources| {
                    resources
                        .xobjects
                        .iter()
                        .filter_map(|(_name, &r)| self.pdf.get(r).ok())
                        .filter_map(|object| match *object {
                            // only images without alpha are supported
                            XObject::Image(ref img) if img.smask.is_none() => {
                                Some(Self::image_from_buf(img))
                            }
                            _ => None,
                        })
                        .collect::<Vec<_>>()
                        .into_iter()
                }
            })
    }

    fn convert_luma1_to_luma8(imdata: &[u8], width: u32) -> Vec<u8> {
        let row_rem = width % 8;
        let row_len = width / 8 + if row_rem != 0 { 1 } else { 0 };

        // convert to luma8
        imdata
            .chunks(row_len as _)
            .flat_map(|chunk| {
                chunk.iter().enumerate().flat_map(|(idx, byte)| {
                    let is_last = idx == row_len as usize - 1;

                    let upper = if is_last { row_rem } else { 8 };

                    (0..upper).rev().map(move |pos| {
                        if byte & (1 << pos) != 0 {
                            u8::MAX
                        } else {
                            u8::MIN
                        }
                    })
                })
            })
            .collect::<Vec<u8>>()
    }

    fn image_from_buf(img: &ImageXObject) -> Result<DynamicImage, PdfError> {
        let buf = img.decode()?;
        let width = img.width as u32;
        let height = img.height as u32;

        let luma_buf = if img.bits_per_component == 1 {
            Self::convert_luma1_to_luma8(&buf, width)
        } else {
            buf.to_vec()
        };

        let imbuf = image::GrayImage::from_raw(width as _, height as _, luma_buf)
            .ok_or(PdfError::ImageConversion)?;

        Ok(DynamicImage::ImageLuma8(imbuf))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert2::{check, let_assert};

    #[test]
    fn extract_luma1() {
        let_assert!(Ok(pdf) = PdfQrExtractor::with_path("testdata/1.pdf"));

        let_assert!(Some(code) = pdf.images().next());

        let_assert!(Ok(code) = code);
    }

    #[test]
    fn extract_luma8() {
        let_assert!(Ok(pdf) = PdfQrExtractor::with_path("testdata/2.pdf"));

        let_assert!(Some(code) = pdf.images().next());

        let_assert!(Ok(code) = code);
    }
}

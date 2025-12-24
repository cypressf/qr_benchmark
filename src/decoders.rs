use anyhow::{anyhow, Result};
use image::DynamicImage;

pub trait QrDecoder {
    fn name(&self) -> &'static str;
    fn decode(&self, image: &DynamicImage) -> Result<String>;
}

pub struct RqrrDecoder;
impl QrDecoder for RqrrDecoder {
    fn name(&self) -> &'static str {
        "rqrr"
    }

    fn decode(&self, image: &DynamicImage) -> Result<String> {
        let gray_image = image.to_luma8();
        let mut img = rqrr::PreparedImage::prepare_from_greyscale(
            gray_image.width() as usize,
            gray_image.height() as usize,
            |x, y| gray_image.get_pixel(x as u32, y as u32)[0],
        );
        
        let grids = img.detect_grids();
        if let Some(grid) = grids.first() {
            let (_meta, content) = grid.decode()?;
            Ok(content)
        } else {
            Err(anyhow!("No QR code detected"))
        }
    }
}

pub struct RxingDecoder;
impl QrDecoder for RxingDecoder {
    fn name(&self) -> &'static str {
        "rxing"
    }

    fn decode(&self, image: &DynamicImage) -> Result<String> {
        let width = image.width();
        let height = image.height();
        let gray_image = image.to_luma8();
        let pixels: Vec<u8> = gray_image.into_raw();
        
        // rxing expects a flat luma buffer
        let result = rxing::helpers::detect_in_luma(pixels, width, height, None)?;
        Ok(result.getText().to_string())
    }
}

pub struct BardecoderDecoder;
impl QrDecoder for BardecoderDecoder {
    fn name(&self) -> &'static str {
        "bardecoder"
    }

    fn decode(&self, image: &DynamicImage) -> Result<String> {
        let decoder = bardecoder::default_decoder();
        
        // bardecoder takes DynamicImage directly
        let results = decoder.decode(image);
        
        // results is Vec<Result<String, Error>>
        if let Some(result) = results.first() {
            match result {
                Ok(content) => Ok(content.clone()),
                Err(e) => Err(anyhow!("Decode error: {:?}", e)),
            }
        } else {
            Err(anyhow!("No QR code detected"))
        }
    }
}

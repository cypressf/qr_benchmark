use anyhow::{anyhow, Result};
use image::DynamicImage;
use rxing::{DecodeHintType, ResultPoint, MultiFormatReader, Reader, BarcodeFormat};
use rxing::common::HybridBinarizer;
use rxing::BinaryBitmap;
use std::collections::{HashMap, HashSet};

pub struct DecodeResult {
    pub text: String,
    // Optional points: [TopLeft, TopRight, BottomRight, BottomLeft]
    pub points: Option<Vec<(f32, f32)>>,
}

pub trait QrDecoder {
    fn name(&self) -> &'static str;
    fn decode(&self, image: &DynamicImage) -> Result<DecodeResult>;
}

pub struct RqrrDecoder;
impl QrDecoder for RqrrDecoder {
    fn name(&self) -> &'static str {
        "rqrr"
    }

    fn decode(&self, image: &DynamicImage) -> Result<DecodeResult> {
        let gray_image = image.to_luma8();
        let mut img = rqrr::PreparedImage::prepare_from_greyscale(
            gray_image.width() as usize,
            gray_image.height() as usize,
            |x, y| gray_image.get_pixel(x as u32, y as u32)[0],
        );
        
        let grids = img.detect_grids();
        for grid in grids {
            if let Ok((_meta, content)) = grid.decode() {
                let corners = grid.bounds.iter().map(|p| (p.x as f32, p.y as f32)).collect();
                return Ok(DecodeResult {
                    text: content,
                    points: Some(corners),
                });
            }
        }
        
        // Fallback: Inverted
        let mut img_inv = rqrr::PreparedImage::prepare_from_greyscale(
            gray_image.width() as usize,
            gray_image.height() as usize,
            |x, y| 255 - gray_image.get_pixel(x as u32, y as u32)[0],
        );
        let grids = img_inv.detect_grids();
        for grid in grids {
            if let Ok((_meta, content)) = grid.decode() {
                let corners = grid.bounds.iter().map(|p| (p.x as f32, p.y as f32)).collect();
                return Ok(DecodeResult {
                    text: content,
                    points: Some(corners),
                });
            }
        }
        
        Err(anyhow!("No QR code detected"))
    }
}

pub struct RxingDecoder;
impl QrDecoder for RxingDecoder {
    fn name(&self) -> &'static str {
        "rxing"
    }

    fn decode(&self, image: &DynamicImage) -> Result<DecodeResult> {
        let width = image.width();
        let height = image.height();
        let gray_image = image.to_luma8();
        let pixels: Vec<u8> = gray_image.into_raw();
        
        let mut hints = HashMap::new();
        hints.insert(DecodeHintType::TRY_HARDER, rxing::DecodeHintValue::TryHarder(true));
        let mut formats = HashSet::new();
        formats.insert(BarcodeFormat::QR_CODE);
        formats.insert(BarcodeFormat::MICRO_QR_CODE);
        hints.insert(DecodeHintType::POSSIBLE_FORMATS, rxing::DecodeHintValue::PossibleFormats(formats));
        hints.insert(DecodeHintType::ALSO_INVERTED, rxing::DecodeHintValue::AlsoInverted(true));

        let source = rxing::Luma8LuminanceSource::new(pixels, width, height);
        let binarizer = HybridBinarizer::new(source);
        let mut bitmap = BinaryBitmap::new(binarizer);
        let mut reader = MultiFormatReader::default();
        
        let result = reader.decode_with_hints(&mut bitmap, &hints)?;
        
        let points = result.getRXingResultPoints().iter().map(|p| (p.getX(), p.getY())).collect();

        Ok(DecodeResult {
            text: result.getText().to_string(),
            points: Some(points),
        })
    }
}

pub struct BardecoderDecoder;
impl QrDecoder for BardecoderDecoder {
    fn name(&self) -> &'static str {
        "bardecoder"
    }

    fn decode(&self, image: &DynamicImage) -> Result<DecodeResult> {
        let decoder = bardecoder::default_decoder();
        
        // bardecoder doesn't easily expose the corner points in the high-level API
        // It returns raw string content.
        // We might need to dig deeper or accept that it doesn't support detection benchmarks well.
        // For now, return None for points.
        
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            decoder.decode(image)
        }));

        match result {
            Ok(results) => {
                if let Some(result) = results.first() {
                    match result {
                        Ok(content) => Ok(DecodeResult {
                            text: content.clone(),
                            points: None, // Bardecoder doesn't expose points in default API
                        }),
                        Err(e) => Err(anyhow!("Decode error: {:?}", e)),
                    }
                } else {
                    Err(anyhow!("No QR code detected"))
                }
            },
            Err(_) => Err(anyhow!("Bardecoder panicked")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_bardecoder_crash_image() {
        let image_path = PathBuf::from("../detection/nominal/image026.jpg");
        if !image_path.exists() {
            eprintln!("Image not found: {:?}", image_path);
            return;
        }

        let img = image::open(&image_path).expect("Failed to open image");
        let decoder = BardecoderDecoder;
        
        let result = decoder.decode(&img);
        
        match result {
            Ok(_) => println!("Decode successful"),
            Err(e) => {
                println!("Decode failed gracefully: {}", e);
                assert_eq!(e.to_string(), "Bardecoder panicked");
            }
        }
    }
}

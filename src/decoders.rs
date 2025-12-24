use anyhow::{anyhow, Result};
use bardecoder::decode::Decode;
use bardecoder::detect::Detect;
use bardecoder::extract::Extract;
use bardecoder::prepare::Prepare;
use image::DynamicImage;
use rxing::common::HybridBinarizer;
use rxing::BinaryBitmap;
use rxing::{BarcodeFormat, DecodeHintType, MultiFormatReader, Reader, ResultPoint};
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
                let corners = grid
                    .bounds
                    .iter()
                    .map(|p| (p.x as f32, p.y as f32))
                    .collect();
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
                let corners = grid
                    .bounds
                    .iter()
                    .map(|p| (p.x as f32, p.y as f32))
                    .collect();
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
        hints.insert(
            DecodeHintType::TRY_HARDER,
            rxing::DecodeHintValue::TryHarder(true),
        );
        let mut formats = HashSet::new();
        formats.insert(BarcodeFormat::QR_CODE);
        formats.insert(BarcodeFormat::MICRO_QR_CODE);
        hints.insert(
            DecodeHintType::POSSIBLE_FORMATS,
            rxing::DecodeHintValue::PossibleFormats(formats),
        );
        hints.insert(
            DecodeHintType::ALSO_INVERTED,
            rxing::DecodeHintValue::AlsoInverted(true),
        );

        let source = rxing::Luma8LuminanceSource::new(pixels, width, height);
        let binarizer = HybridBinarizer::new(source);
        let mut bitmap = BinaryBitmap::new(binarizer);
        let mut reader = MultiFormatReader::default();

        let result = reader.decode_with_hints(&mut bitmap, &hints)?;

        let points = result
            .getRXingResultPoints()
            .iter()
            .map(|p| (p.getX(), p.getY()))
            .collect();

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
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            // Manually build pipeline to get location info
            let prepare = bardecoder::prepare::BlockedMean::new(5, 7);
            let prepared = prepare.prepare(image);

            let detect = bardecoder::detect::LineScan::new();
            let locations = detect.detect(&prepared);

            if locations.is_empty() {
                return Err(anyhow!("No QR code detected"));
            }

            // Try to decode all locations, return first success
            for loc in locations {
                let bardecoder::detect::Location::QR(qr_loc) = loc;

                // Extract points before moving qr_loc
                let p1 = (qr_loc.top_left.x as f32, qr_loc.top_left.y as f32);
                let p2 = (qr_loc.top_right.x as f32, qr_loc.top_right.y as f32);
                let p3 = (qr_loc.bottom_left.x as f32, qr_loc.bottom_left.y as f32);

                let version = qr_loc.version;
                let modules = 17 + 4 * version;
                let distance_modules = modules as f32 - 7.0;

                let vec_right = (p2.0 - p1.0, p2.1 - p1.1);
                let vec_down = (p3.0 - p1.0, p3.1 - p1.1);

                // Unit vectors (per module)
                let unit_right = (
                    vec_right.0 / distance_modules,
                    vec_right.1 / distance_modules,
                );
                let unit_down = (vec_down.0 / distance_modules, vec_down.1 / distance_modules);

                // Extrapolate to corners (centers are at 3.5, 3.5 relative to corners)
                // p1 is TL center. Corner is -3.5 u, -3.5 v
                let p1_corner = (
                    p1.0 - 3.5 * unit_right.0 - 3.5 * unit_down.0,
                    p1.1 - 3.5 * unit_right.1 - 3.5 * unit_down.1,
                );

                // p2 is TR center. Corner is +3.5 u, -3.5 v
                let p2_corner = (
                    p2.0 + 3.5 * unit_right.0 - 3.5 * unit_down.0,
                    p2.1 + 3.5 * unit_right.1 - 3.5 * unit_down.1,
                );

                // p3 is BL center. Corner is -3.5 u, +3.5 v
                let p3_corner = (
                    p3.0 - 3.5 * unit_right.0 + 3.5 * unit_down.0,
                    p3.1 - 3.5 * unit_right.1 + 3.5 * unit_down.1,
                );

                // p4 is BR corner. Parallelogram completion.
                let p4_corner = (
                    p2_corner.0 + (p3_corner.0 - p1_corner.0),
                    p2_corner.1 + (p3_corner.1 - p1_corner.1),
                );

                let points = vec![p1_corner, p2_corner, p4_corner, p3_corner]; // Order: TL, TR, BR, BL

                let extract = bardecoder::extract::QRExtractor::new();
                let decode = bardecoder::decode::QRDecoder::new();

                let extracted = extract.extract(&prepared, qr_loc);
                let decoded = decode.decode(extracted);

                match decoded {
                    Ok(content) => {
                        return Ok(DecodeResult {
                            text: content,
                            points: Some(points),
                        });
                    }
                    Err(_) => continue, // Try next location
                }
            }

            Err(anyhow!("No QR code detected"))
        }));

        match result {
            Ok(res) => res,
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

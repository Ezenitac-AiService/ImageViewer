//! Pure-Rust 13-field EXIF metadata extractor (`kamadak-exif`).

use crate::cache::MetadataItem;
use exif::{In, Reader, Tag, Value};
use std::io::Cursor;

pub struct ExifExtractor;

impl ExifExtractor {
    /// Extracts up to 13 documented EXIF fields, failing silently on corrupt or missing tags.
    pub fn extract(bytes: &[u8]) -> Vec<MetadataItem> {
        let mut cursor = Cursor::new(bytes);
        let Ok(exif) = Reader::new().read_from_container(&mut cursor) else {
            return Vec::new();
        };

        let mut items = Vec::new();

        // 1. Camera (Make + Model)
        let make = get_ascii(&exif, Tag::Make);
        let model = get_ascii(&exif, Tag::Model);
        let camera = match (make, model) {
            (Some(mk), Some(md)) => {
                if md.to_ascii_lowercase().contains(&mk.to_ascii_lowercase()) {
                    Some(md)
                } else {
                    Some(format!("{mk} {md}"))
                }
            }
            (Some(mk), None) => Some(mk),
            (None, Some(md)) => Some(md),
            (None, None) => None,
        };
        push_if(&mut items, "exif.camera", camera);

        // 2. Lens
        push_if(&mut items, "exif.lens", get_ascii(&exif, Tag::LensModel));
        // 3. Date Taken
        push_if(
            &mut items,
            "exif.date_taken",
            get_ascii(&exif, Tag::DateTimeOriginal).or_else(|| get_ascii(&exif, Tag::DateTime)),
        );
        // 4. Exposure Time
        if let Some(f) = exif.get_field(Tag::ExposureTime, In::PRIMARY) {
            items.push(MetadataItem {
                label_key: "exif.exposure_time",
                value: f.display_value().with_unit(&exif).to_string(),
            });
        }
        // 5. Aperture
        if let Some(f) = exif.get_field(Tag::FNumber, In::PRIMARY) {
            items.push(MetadataItem {
                label_key: "exif.aperture",
                value: format!("f/{}", f.display_value()),
            });
        }
        // 6. ISO
        if let Some(f) = exif.get_field(Tag::PhotographicSensitivity, In::PRIMARY) {
            items.push(MetadataItem {
                label_key: "exif.iso",
                value: f.display_value().to_string(),
            });
        }
        // 7. Focal Length (+ 35mm equiv)
        if let Some(f) = exif.get_field(Tag::FocalLength, In::PRIMARY) {
            let mut val = f.display_value().with_unit(&exif).to_string();
            if let Some(eq) = exif.get_field(Tag::FocalLengthIn35mmFilm, In::PRIMARY) {
                val.push_str(&format!(" ({}mm eq.)", eq.display_value()));
            }
            items.push(MetadataItem {
                label_key: "exif.focal_length",
                value: val,
            });
        }
        // 8. Exposure Bias
        if let Some(f) = exif.get_field(Tag::ExposureBiasValue, In::PRIMARY) {
            items.push(MetadataItem {
                label_key: "exif.exposure_bias",
                value: f.display_value().with_unit(&exif).to_string(),
            });
        }
        // 9. Flash
        if let Some(f) = exif.get_field(Tag::Flash, In::PRIMARY) {
            items.push(MetadataItem {
                label_key: "exif.flash",
                value: f.display_value().to_string(),
            });
        }
        // 10. GPS
        if let Some(gps) = extract_gps(&exif) {
            items.push(MetadataItem {
                label_key: "exif.gps",
                value: gps,
            });
        }
        // 11. Software
        push_if(&mut items, "exif.software", get_ascii(&exif, Tag::Software));
        // 12. Artist
        push_if(&mut items, "exif.artist", get_ascii(&exif, Tag::Artist));
        // 13. Copyright
        push_if(&mut items, "exif.copyright", get_ascii(&exif, Tag::Copyright));

        items
    }

    /// Reads EXIF Orientation tag (`1..=8`) if present.
    pub fn read_orientation_tag(bytes: &[u8]) -> u16 {
        let mut cursor = Cursor::new(bytes);
        let Ok(exif) = Reader::new().read_from_container(&mut cursor) else {
            return 1;
        };
        exif.get_field(Tag::Orientation, In::PRIMARY)
            .and_then(|f| match &f.value {
                Value::Short(v) => v.first().copied(),
                _ => None,
            })
            .unwrap_or(1)
    }
}

fn get_ascii(exif: &exif::Exif, tag: Tag) -> Option<String> {
    let f = exif.get_field(tag, In::PRIMARY)?;
    let s = f
        .display_value()
        .to_string()
        .trim_matches('"')
        .trim()
        .to_string();
    if s.is_empty() { None } else { Some(s) }
}

fn push_if(items: &mut Vec<MetadataItem>, label_key: &'static str, val: Option<String>) {
    if let Some(value) = val {
        items.push(MetadataItem { label_key, value });
    }
}

fn extract_gps(exif: &exif::Exif) -> Option<String> {
    let lat = dms_to_decimal(exif.get_field(Tag::GPSLatitude, In::PRIMARY)?)?;
    let lon = dms_to_decimal(exif.get_field(Tag::GPSLongitude, In::PRIMARY)?)?;
    let lat_ref = get_ascii(exif, Tag::GPSLatitudeRef).unwrap_or_else(|| "N".to_string());
    let lon_ref = get_ascii(exif, Tag::GPSLongitudeRef).unwrap_or_else(|| "E".to_string());
    let lat_signed = if lat_ref.starts_with('S') { -lat } else { lat };
    let lon_signed = if lon_ref.starts_with('W') { -lon } else { lon };
    Some(format!("{lat_signed:.5}, {lon_signed:.5}"))
}

fn dms_to_decimal(field: &exif::Field) -> Option<f64> {
    if let Value::Rational(rats) = &field.value
        && rats.len() >= 3
        && rats[0].denom != 0
        && rats[1].denom != 0
        && rats[2].denom != 0
    {
        let deg = f64::from(rats[0].num) / f64::from(rats[0].denom);
        let min = f64::from(rats[1].num) / f64::from(rats[1].denom);
        let sec = f64::from(rats[2].num) / f64::from(rats[2].denom);
        return Some(deg + min / 60.0 + sec / 3600.0);
    }
    None
}

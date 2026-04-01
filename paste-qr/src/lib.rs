//! paste-qr — ZOS plugin for QR code generation.
//!
//! Commands:
//!   qr-url <url>       → QR PNG of the URL
//!   qr-data <text>     → QR PNG of arbitrary text/data
//!   qr-svg <text>      → QR as SVG string

use qrcode::render::svg;
use qrcode::QrCode;
use std::io::Cursor;

/// Generate QR code as PNG bytes.
pub fn qr_png(data: &str) -> Result<Vec<u8>, String> {
    let code = QrCode::new(data.as_bytes()).map_err(|e| e.to_string())?;
    let img = code
        .render::<image::Luma<u8>>()
        .quiet_zone(true)
        .min_dimensions(256, 256)
        .build();
    let mut buf = Cursor::new(Vec::new());
    img.write_to(&mut buf, image::ImageFormat::Png)
        .map_err(|e| e.to_string())?;
    Ok(buf.into_inner())
}

/// Generate QR code as SVG string.
pub fn qr_svg(data: &str) -> Result<String, String> {
    let code = QrCode::new(data.as_bytes()).map_err(|e| e.to_string())?;
    Ok(code
        .render::<svg::Color>()
        .quiet_zone(true)
        .min_dimensions(256, 256)
        .build())
}

/// Plugin execute: returns JSON with base64 PNG or SVG string.
pub fn execute(command: &str, args: &[String]) -> Result<serde_json::Value, String> {
    let data = args.first().ok_or("missing argument")?;
    match command {
        "qr-url" | "qr-data" => {
            let png = qr_png(data)?;
            Ok(serde_json::json!({
                "format": "png",
                "size": png.len(),
                "base64": base64_encode(&png),
            }))
        }
        "qr-svg" => {
            let svg = qr_svg(data)?;
            Ok(serde_json::json!({
                "format": "svg",
                "size": svg.len(),
                "svg": svg,
            }))
        }
        _ => Err(format!("unknown command: {command}")),
    }
}

fn base64_encode(data: &[u8]) -> String {
    const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::new();
    for chunk in data.chunks(3) {
        let b = match chunk.len() {
            3 => [chunk[0], chunk[1], chunk[2]],
            2 => [chunk[0], chunk[1], 0],
            1 => [chunk[0], 0, 0],
            _ => unreachable!(),
        };
        let n = (b[0] as u32) << 16 | (b[1] as u32) << 8 | b[2] as u32;
        out.push(CHARS[(n >> 18 & 63) as usize] as char);
        out.push(CHARS[(n >> 12 & 63) as usize] as char);
        if chunk.len() > 1 {
            out.push(CHARS[(n >> 6 & 63) as usize] as char);
        } else {
            out.push('=');
        }
        if chunk.len() > 2 {
            out.push(CHARS[(n & 63) as usize] as char);
        } else {
            out.push('=');
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_qr_png() {
        let png = qr_png("https://solana.solfunmeme.com/pastebin/").unwrap();
        assert!(png.len() > 100);
        assert_eq!(&png[..4], &[0x89, 0x50, 0x4E, 0x47]); // PNG magic
    }

    #[test]
    fn test_qr_svg() {
        let svg = qr_svg("test data").unwrap();
        assert!(svg.contains("<svg"));
        assert!(svg.contains("</svg>"));
    }

    #[test]
    fn test_execute_url() {
        let result = execute("qr-url", &["https://example.com".to_string()]).unwrap();
        assert_eq!(result["format"], "png");
        assert!(result["size"].as_u64().unwrap() > 0);
    }

    #[test]
    fn test_execute_svg() {
        let result = execute("qr-svg", &["hello".to_string()]).unwrap();
        assert_eq!(result["format"], "svg");
        assert!(result["svg"].as_str().unwrap().contains("<svg"));
    }

    #[test]
    fn test_execute_missing_arg() {
        let result = execute("qr-url", &[]);
        assert!(result.is_err());
    }
}

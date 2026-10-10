use anyhow::{Result, bail};
use serde_json::{Value, json};

fn luminance(color: &str) -> Result<f64> {
    if color.len() != 7
        || !color.starts_with('#')
        || !color.as_bytes()[1..].iter().all(u8::is_ascii_hexdigit)
    {
        bail!("use an opaque six-digit sRGB color such as #345947");
    }
    let mut value = 0.0;
    for (index, weight) in [0.2126, 0.7152, 0.0722].iter().enumerate() {
        let start = 1 + index * 2;
        let c = f64::from(u8::from_str_radix(&color[start..start + 2], 16)?) / 255.0;
        value += weight
            * if c <= 0.04045 {
                c / 12.92
            } else {
                ((c + 0.055) / 1.055).powf(2.4)
            };
    }
    Ok(value)
}

pub fn contrast(foreground: &str, background: &str) -> Result<Value> {
    let a = luminance(foreground)?;
    let b = luminance(background)?;
    let ratio = (a.max(b) + 0.05) / (a.min(b) + 0.05);
    Ok(
        json!({"foreground":foreground,"background":background,"ratio":ratio,
        "meets_4_5":ratio >= 4.5,"meets_3":ratio >= 3.0,
        "scope":"Opaque sRGB pair only. Interpret thresholds for the actual text or component; inspect gradients, transparency and rendered states separately."}),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn known_ratios_and_input_boundaries() {
        assert_eq!(contrast("#ffffff", "#000000").unwrap()["ratio"], 21.0);
        assert_eq!(contrast("#345947", "#345947").unwrap()["ratio"], 1.0);
        assert!(contrast("#ffffff80", "#000000").is_err());
        assert!(contrast("#☃abc", "#000000").is_err());
    }
}

"""Measure one pair of opaque sRGB hex colors; no whole-page compliance claim."""
import argparse
import json
import re


def luminance(value):
    if not re.fullmatch(r"#[0-9a-fA-F]{6}", value):
        raise ValueError("Use an opaque six-digit sRGB hex color, for example #345947")
    channels = [int(value[i:i + 2], 16) / 255 for i in (1, 3, 5)]
    linear = [c / 12.92 if c <= 0.04045 else ((c + 0.055) / 1.055) ** 2.4
              for c in channels]
    return sum(c * w for c, w in zip(linear, (0.2126, 0.7152, 0.0722)))


def contrast(first, second):
    light, dark = sorted((luminance(first), luminance(second)), reverse=True)
    return (light + 0.05) / (dark + 0.05)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("foreground")
    parser.add_argument("background")
    args = parser.parse_args()
    try:
        ratio = contrast(args.foreground, args.background)
    except ValueError as error:
        parser.error(str(error))
    print(json.dumps({"foreground": args.foreground, "background": args.background,
                      "ratio": ratio, "meets_4_5": ratio >= 4.5,
                      "meets_3": ratio >= 3,
                      "limit": "Opaque pair only; interpret thresholds for the actual text/component."},
                     indent=2))


if __name__ == "__main__":
    main()

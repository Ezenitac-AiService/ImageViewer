using System.Globalization;
using ImageMagick;
using ImageViewer.Models;

namespace ImageViewer.Services;

/// <summary>
/// 정보 패널에 보여 줄 EXIF 항목을 읽는다. 값이 없는 항목은 건너뛴다.
/// </summary>
public static class ExifReader
{
    public static IReadOnlyList<MetadataItem> Read(IMagickImage<byte> image)
    {
        var items = new List<MetadataItem>();
        IExifProfile? exif;
        try
        {
            exif = image.GetExifProfile();
        }
        catch (MagickException)
        {
            return items;
        }

        if (exif is null)
        {
            return items;
        }

        void Add(string label, string? value)
        {
            if (!string.IsNullOrWhiteSpace(value))
            {
                items.Add(new MetadataItem(label, value.Trim().TrimEnd('\0')));
            }
        }

        try
        {
            var make = exif.GetValue(ExifTag.Make)?.Value?.Trim();
            var model = exif.GetValue(ExifTag.Model)?.Value?.Trim();
            Add("카메라", JoinCamera(make, model));
            Add("렌즈", exif.GetValue(ExifTag.LensModel)?.Value);
            Add("촬영 일시", FormatExifDate(exif.GetValue(ExifTag.DateTimeOriginal)?.Value ?? exif.GetValue(ExifTag.DateTime)?.Value));

            if (exif.GetValue(ExifTag.ExposureTime)?.Value is { } exposure)
            {
                Add("노출 시간", FormatExposure(exposure));
            }

            if (exif.GetValue(ExifTag.FNumber)?.Value is { } fNumber && fNumber.Denominator != 0)
            {
                Add("조리개", "f/" + fNumber.ToDouble().ToString("0.#", CultureInfo.InvariantCulture));
            }

            if (exif.GetValue(ExifTag.ISOSpeedRatings)?.Value is { Length: > 0 } iso)
            {
                Add("ISO", iso[0].ToString(CultureInfo.InvariantCulture));
            }

            if (exif.GetValue(ExifTag.FocalLength)?.Value is { } focal && focal.Denominator != 0)
            {
                var text = focal.ToDouble().ToString("0.#", CultureInfo.InvariantCulture) + " mm";
                if (exif.GetValue(ExifTag.FocalLengthIn35mmFilm)?.Value is { } focal35 && focal35 > 0)
                {
                    text += $" (35mm 환산 {focal35} mm)";
                }

                Add("초점 거리", text);
            }

            if (exif.GetValue(ExifTag.ExposureBiasValue)?.Value is { } bias && bias.Denominator != 0)
            {
                Add("노출 보정", bias.ToDouble().ToString("+0.0;-0.0;0", CultureInfo.InvariantCulture) + " EV");
            }

            if (exif.GetValue(ExifTag.Flash)?.Value is { } flash)
            {
                Add("플래시", (flash & 0x1) != 0 ? "사용" : "사용 안 함");
            }

            Add("GPS", FormatGps(exif));
            Add("소프트웨어", exif.GetValue(ExifTag.Software)?.Value);
            Add("작성자", exif.GetValue(ExifTag.Artist)?.Value);
            Add("저작권", exif.GetValue(ExifTag.Copyright)?.Value);
        }
        catch (Exception ex) when (ex is MagickException or InvalidCastException or ArgumentException)
        {
            // 손상된 EXIF 는 읽을 수 있는 데까지만 표시한다.
        }

        return items;
    }

    private static string? JoinCamera(string? make, string? model)
    {
        if (string.IsNullOrEmpty(model))
        {
            return make;
        }

        if (string.IsNullOrEmpty(make) || model.StartsWith(make, StringComparison.OrdinalIgnoreCase))
        {
            return model;
        }

        return $"{make} {model}";
    }

    internal static string? FormatExifDate(string? value)
    {
        if (string.IsNullOrWhiteSpace(value))
        {
            return null;
        }

        // EXIF 형식: "yyyy:MM:dd HH:mm:ss"
        return DateTime.TryParseExact(value.Trim().TrimEnd('\0'), "yyyy:MM:dd HH:mm:ss",
            CultureInfo.InvariantCulture, DateTimeStyles.None, out var date)
            ? date.ToString("yyyy-MM-dd HH:mm:ss", CultureInfo.InvariantCulture)
            : value;
    }

    internal static string FormatExposure(Rational exposure)
    {
        if (exposure.Denominator == 0)
        {
            return "-";
        }

        var seconds = exposure.ToDouble();
        if (seconds >= 1 || seconds <= 0)
        {
            return seconds.ToString("0.#", CultureInfo.InvariantCulture) + " 초";
        }

        return $"1/{Math.Round(1 / seconds):0} 초";
    }

    private static string? FormatGps(IExifProfile exif)
    {
        var lat = exif.GetValue(ExifTag.GPSLatitude)?.Value;
        var lon = exif.GetValue(ExifTag.GPSLongitude)?.Value;
        if (lat is not { Length: 3 } || lon is not { Length: 3 })
        {
            return null;
        }

        var latRef = exif.GetValue(ExifTag.GPSLatitudeRef)?.Value;
        var lonRef = exif.GetValue(ExifTag.GPSLongitudeRef)?.Value;
        var latitude = ToDegrees(lat) * (string.Equals(latRef, "S", StringComparison.OrdinalIgnoreCase) ? -1 : 1);
        var longitude = ToDegrees(lon) * (string.Equals(lonRef, "W", StringComparison.OrdinalIgnoreCase) ? -1 : 1);
        return string.Create(CultureInfo.InvariantCulture, $"{latitude:0.000000}, {longitude:0.000000}");
    }

    private static double ToDegrees(Rational[] dms)
    {
        static double V(Rational r) => r.Denominator == 0 ? 0 : r.ToDouble();
        return V(dms[0]) + V(dms[1]) / 60 + V(dms[2]) / 3600;
    }
}

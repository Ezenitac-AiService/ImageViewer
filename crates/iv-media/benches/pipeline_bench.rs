use criterion::{black_box, criterion_group, criterion_main, Criterion};
use iv_media::decode::{DecodeOptions, MediaDecoder};

fn build_synthetic_jpeg(w: u32, h: u32) -> Vec<u8> {
    let img = image::RgbImage::from_fn(w, h, |x, y| {
        image::Rgb([(x % 256) as u8, (y % 256) as u8, ((x + y) % 256) as u8])
    });
    let mut out = Vec::new();
    image::DynamicImage::ImageRgb8(img)
        .write_to(&mut std::io::Cursor::new(&mut out), image::ImageFormat::Jpeg)
        .expect("encode bench jpeg");
    out
}

fn bench_pipeline_latency(c: &mut Criterion) {
    let jpeg = build_synthetic_jpeg(2592, 1728);
    let opts_windowed = DecodeOptions {
        max_megapixels: 150,
        target_viewport_physical: Some((1280, 800)),
        page_index: 0,
    };
    let opts_fullscreen = DecodeOptions {
        max_megapixels: 150,
        target_viewport_physical: Some((2560, 1440)),
        page_index: 0,
    };

    c.bench_function("decode_resize_2592x1728_windowed_1280x800", |b| {
        b.iter(|| {
            let frame = MediaDecoder::decode_bytes(black_box(&jpeg), "bench.jpg", &opts_windowed)
                .expect("decode");
            black_box(frame.buffer_width)
        });
    });

    c.bench_function("decode_resize_2592x1728_fullscreen_2560x1440", |b| {
        b.iter(|| {
            let frame = MediaDecoder::decode_bytes(black_box(&jpeg), "bench.jpg", &opts_fullscreen)
                .expect("decode");
            black_box(frame.buffer_width)
        });
    });
}

criterion_group!(benches, bench_pipeline_latency);
criterion_main!(benches);

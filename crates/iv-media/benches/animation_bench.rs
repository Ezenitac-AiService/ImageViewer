use criterion::{black_box, criterion_group, criterion_main, Criterion};
use iv_media::animation::AnimationPlayer;

fn build_synthetic_gif(frames: usize, w: u32, h: u32) -> Vec<u8> {
    let mut out = Vec::new();
    let mut enc = image::codecs::gif::GifEncoder::new(&mut out);
    for i in 0..frames {
        let px = image::Rgba([(i % 256) as u8, 64, 128, 255]);
        enc.encode_frame(image::Frame::new(image::RgbaImage::from_pixel(w, h, px)))
            .expect("encode gif frame");
    }
    drop(enc);
    out
}

fn bench_animation_first_frame(c: &mut Criterion) {
    let gif = build_synthetic_gif(30, 320, 240);
    c.bench_function("animation_decode_30f_320x240", |b| {
        b.iter(|| {
            let player = AnimationPlayer::from_bytes(black_box(&gif), "bench.gif").expect("anim");
            black_box(player.frame_count())
        });
    });
}

criterion_group!(benches, bench_animation_first_frame);
criterion_main!(benches);

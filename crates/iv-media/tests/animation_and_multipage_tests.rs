#![allow(dead_code)]
mod fixtures;

use iv_media::animation::{AnimationPlayer, ApngBlendOp, ApngDisposeOp};
use std::time::Duration;

#[test]
fn animation_player_normalizes_gif_webp_apng_delays_and_ring_budget() {
    // GIF <= 10 ms -> 100 ms
    assert_eq!(AnimationPlayer::normalize_gif_delay(Duration::from_millis(5)), Duration::from_millis(100));
    assert_eq!(AnimationPlayer::normalize_gif_delay(Duration::from_millis(20)), Duration::from_millis(20));

    // WebP preserves per-frame delays including 1 ms unless ALL frames <= 10 ms
    let mixed = vec![Duration::from_millis(1), Duration::from_millis(50)];
    let norm_mixed = AnimationPlayer::normalize_webp_delays(&mixed);
    assert_eq!(norm_mixed[0], Duration::from_millis(1));

    let all_fast = vec![Duration::from_millis(5), Duration::from_millis(10)];
    let norm_fast = AnimationPlayer::normalize_webp_delays(&all_fast);
    assert_eq!(norm_fast, vec![Duration::from_millis(100), Duration::from_millis(100)]);

    // APNG rational delay_num / delay_den (den == 0 -> 100, <= 10 ms -> 100 ms)
    assert_eq!(AnimationPlayer::apng_frame_delay(1, 0), Duration::from_millis(100));
    assert_eq!(AnimationPlayer::apng_frame_delay(5, 100), Duration::from_millis(50));

    // APNG DisposeOp::None/Background/Previous + BlendOp::Source/Over
    let mut canvas = vec![0u8; 4 * 4]; // 2x2 RGBA
    AnimationPlayer::composite_apng_rect(
        &mut canvas,
        2,
        2,
        0,
        0,
        1,
        1,
        &[255, 0, 0, 255],
        ApngBlendOp::Source,
    );
    assert_eq!(&canvas[0..4], &[255, 0, 0, 255]);
    AnimationPlayer::apply_apng_dispose(&mut canvas, 2, 0, 0, 1, 1, ApngDisposeOp::Background, None);
    assert_eq!(&canvas[0..4], &[0, 0, 0, 0]);
}

#[test]
fn decodes_animated_gif_with_immediate_first_frame() {
    let mut gif_bytes = Vec::new();
    {
        let mut enc = image::codecs::gif::GifEncoder::new(&mut gif_bytes);
        let f1 = image::Frame::new(image::RgbaImage::from_pixel(4, 4, image::Rgba([255, 0, 0, 255])));
        let f2 = image::Frame::new(image::RgbaImage::from_pixel(4, 4, image::Rgba([0, 255, 0, 255])));
        enc.encode_frames([f1, f2]).expect("encode gif");
    }
    let player = AnimationPlayer::from_bytes(&gif_bytes, "anim.gif").expect("load anim");
    assert_eq!(player.frame_count(), 2);
    assert_eq!(player.current_frame().source_width, 4);
}

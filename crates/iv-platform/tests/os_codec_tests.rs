#![cfg(feature = "os-codecs")]
use iv_core::codecs::{OsCodecError, OsCodecProvider};
use iv_platform::codecs::HostOsCodecProvider;

#[test]
fn os_codec_provider_rejects_script_headers_and_oversized_streams_fail_closed() {
    let provider = HostOsCodecProvider;
    assert!(provider.supports_extension("heic"));
    assert!(provider.supports_extension("dng"));
    assert!(!provider.supports_extension("png"));

    let script = b"  <svg onload=alert(1)>";
    let err = provider
        .decode_guarded(script, "heic", 150)
        .expect_err("must block script polyglot");
    assert_eq!(err, OsCodecError::SecurityRejected);
}

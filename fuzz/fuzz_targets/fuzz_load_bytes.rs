#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    // The fuzzer does not know or care what valid .vox data looks like.
    // Therefore, this fuzz test can never _expect success;_ only expect failures to
    // be reported via `Err` instead of panic.
    let _ = dot_vox::load_bytes(data);
});

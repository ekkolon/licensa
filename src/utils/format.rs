use std::time::Instant;

pub fn elapsed_time_in_secs(time: Instant) -> String {
    let secs = time.elapsed().as_secs_f32();
    let mut secs_rounded = secs * 100.0;
    secs_rounded = f32::floor(secs_rounded);
    secs_rounded /= 100.0;
    format!("{secs_rounded}s")
}

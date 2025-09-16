use rt_attrs::{non_realtime, realtime};

/// Realtime function: fast audio processing
#[realtime]
pub fn do_rt() {
    // Realtime function: only lightweight operations
    println!("Execute realtime function: do_rt");
}

/// Non-realtime function: requires memory allocation
#[non_realtime("alloc")]
pub fn do_slow() {
    let _ = Vec::<u8>::with_capacity(1024);
    println!("Execute non-realtime function: do_slow (performed memory allocation)");
}

/// Realtime function A
#[realtime]
pub fn realtime_a() {
    println!("Execute realtime function A: realtime_a");
}

/// Realtime function B
#[realtime]
pub fn realtime_b() {
    println!("Execute realtime function B: realtime_b");
}

/// Realtime function C
#[realtime]
pub fn realtime_c() {
    println!("Execute realtime function C: realtime_c");
}

/// Realtime function D
#[realtime]
pub fn realtime_d() {
    println!("Execute realtime function D: realtime_d");
}

/// Realtime function E
#[realtime]
pub fn realtime_e() {
    println!("Execute realtime function E: realtime_e");
}

/// Non-realtime function A: I/O operations
#[non_realtime("io")]
pub fn non_realtime_a() {
    println!("Execute non-realtime function A: non_realtime_a");
}

/// Non-realtime function B: complex computation
#[non_realtime("complex_calc")]
pub fn non_realtime_b() {
    println!("Execute non-realtime function B: non_realtime_b");
}

/// Non-realtime function C: network operations
#[non_realtime("network")]
pub fn non_realtime_c() {
    println!("Execute non-realtime function C: non_realtime_c");
}

/// Non-realtime function D: file operations
#[non_realtime("file_io")]
pub fn non_realtime_d() {
    println!("Execute non-realtime function D: non_realtime_d");
}

/// Non-realtime function E: database operations
#[non_realtime("database")]
pub fn non_realtime_e() {
    println!("Execute non-realtime function E: non_realtime_e");
}

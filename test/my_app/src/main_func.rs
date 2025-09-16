// src/main.rs
use rt_attrs::realtime;
use my_functions_lib::{do_rt, do_slow, realtime_d, non_realtime_d, realtime_e, non_realtime_e};

// #[realtime]
// fn audio() {
//     do_rt();    // OK - realtime function calling a realtime function
//     do_slow();  // ❌ Dylint should warn: realtime function calling a non-realtime function
// }

#[realtime]
fn A()
{
    print!("rt:realtime");
}


fn B() {
    print!("rt:realtime");
}

#[realtime]
fn test_realtime_call_nonrealtime() {
    A();
    B();
}

fn main() {    
    println!("\n=== Direct function calls ===");
    realtime_e();
    non_realtime_e();
}

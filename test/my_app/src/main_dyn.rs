// my_app/src/main.rs

// Nightly: attributes on statements/expressions & proc-macro hygiene adjustments
#![feature(stmt_expr_attributes)]
#![feature(proc_macro_hygiene)]
#![feature(register_tool)]
#![register_tool(dylint)]

use std::f64::consts::PI;
use shape_macros::assert_dyn_type; // your marker macro

// ---- Trait & types ----
trait Shape {
    fn name(&self) -> &'static str;
    fn area(&self) -> f64;
}

struct Circle { r: f64 }
struct Rect { w: f64, h: f64 }

impl Shape for Circle {
    fn name(&self) -> &'static str { "Circle" }
    fn area(&self) -> f64 { PI * self.r * self.r }
}
impl Shape for Rect {
    fn name(&self) -> &'static str { "Rect" }
    fn area(&self) -> f64 { self.w * self.h }
}


// Function pointer target
fn via_fn_ptr(s: &dyn Shape) -> f64 {
    s.area()
}

fn main() {
    let mut shapes: Vec<Box<dyn Shape>> = Vec::new();
    shapes.push(Box::new(Circle { r: 2.0 }));
    shapes.push(Box::new(Rect { w: 3.0, h: 4.0 }));

    // ========== 1) trait dispatch (dynamic) ==========
    #[assert_dyn_type("trait_dyn", "realtime")]
    let s: &dyn Shape = shapes[0].as_ref(); // &Box<dyn Shape> -> &dyn Shape
    println!("(dyn dispatch) {} area = {:.2}", s.name(), s.area());


    // ========== 2) function pointer call ==========
    #[assert_dyn_type("fn_ptr", "realtime")]
    let fp: fn(&dyn Shape) -> f64 = via_fn_ptr; // function pointer
    println!("(fn ptr) area = {:.2}", fp(s));

    // ========== 3) closure call ==========
    #[assert_dyn_type("closure", "realtime")]
    let print_with_closure = |shape: &dyn Shape| {
        println!("(closure) {} area = {:.2}", shape.name(), shape.area());
    };
    print_with_closure(s);
}

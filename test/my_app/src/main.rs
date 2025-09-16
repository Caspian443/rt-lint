// Nightly：语句/表达式上的属性 & 过程宏卫生调整
#![feature(stmt_expr_attributes)]
#![feature(proc_macro_hygiene)]

use rt_attrs::rt_call_info;
use rt_attrs::{non_realtime, realtime};

// 定义一个音频处理 trait
trait AudioProcessor {
    #[non_realtime]
    fn process_audio(&self, _buffer: &mut [f32]) {}

    fn get_name(&self) -> &str;
}

struct SimpleProcessor;

impl AudioProcessor for SimpleProcessor {
    fn process_audio(&self, _buffer: &mut [f32]) {}

    fn get_name(&self) -> &str {
        "SimpleProcessor"
    }
}
#[non_realtime]
fn print_rt(name: &str, area: f32) {
    println!("[RT] {} {:.2}", name, area);
}

#[realtime]
fn main() {
    let processor = SimpleProcessor;
    let mut buffer = vec![0.0f32; 1024];

    // 调用 trait 方法
    processor.process_audio(&mut buffer);

    // 基础数据，无需 trait
    let name = "circle";
    let area = 3.14_f32;

    print_rt(name, area);

    // ========== 1) 闭包调用（realtime） ==========
    #[rt_call_info("closure", "non_realtime")]
    let print_with_closure = |name: &str, area: f32| {
        println!("(closure/realtime) {} area = {:.2}", name, area);
    };
    print_with_closure(name, area);

    let fp: fn(&str, f32) = print_rt; // RHS 是函数项路径，dylint 解析 DefId 检查 doc 属性
    fp(name, area);

    // ========== 3) 变量传播：让 other_fp 继承 fp 的实时语义 ==========
    let other_fp = fp;
    other_fp(name, area);
}

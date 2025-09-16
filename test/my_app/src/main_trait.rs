// Simple trait test example
use rt_attrs::realtime;

// Define an audio processing trait
trait AudioProcessor {
    #[realtime]
    fn process_audio(&self, _buffer: &mut [f32]){

    }
    
    fn get_name(&self) -> &str;
}

// Audio processor implementation
struct SimpleProcessor;

impl AudioProcessor for SimpleProcessor {
    fn process_audio(&self, _buffer: &mut [f32]) {
        println!("Processing audio buffer");
    }
    
    fn get_name(&self) -> &str {
        "Simple Processor"
    }
}


fn main() {

    let processor = SimpleProcessor;
    let mut buffer = vec![0.0f32; 1024];
    
    // Call trait method
    processor.process_audio(&mut buffer);

    // A();
}

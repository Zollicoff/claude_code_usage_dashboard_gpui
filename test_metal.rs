use metal::*;

fn main() {
    println!("Testing Metal GPU availability...");
    
    match Device::system_default() {
        Some(device) => {
            println!("✅ Metal device found: {}", device.name());
            println!("   - Max buffer length: {}", device.max_buffer_length());
            println!("   - Low power: {}", device.is_low_power());
            
            // Try to compile a simple shader
            let shader_source = r#"
                #include <metal_stdlib>
                using namespace metal;
                
                kernel void test_kernel(device float* data [[buffer(0)]],
                                      uint index [[thread_position_in_grid]]) {
                    data[index] = data[index] * 2.0;
                }
            "#;
            
            match device.new_library_with_source(shader_source, &CompileOptions::new()) {
                Ok(library) => {
                    println!("✅ Metal shader compilation successful!");
                    
                    match library.get_function("test_kernel", None) {
                        Ok(_) => println!("✅ Metal function 'test_kernel' found!"),
                        Err(e) => println!("❌ Metal function not found: {:?}", e),
                    }
                },
                Err(e) => println!("❌ Metal shader compilation failed: {:?}", e),
            }
        },
        None => {
            println!("❌ No Metal device available");
        }
    }
}
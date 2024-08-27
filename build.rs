use std::fs;
use std::path::Path;
use std::process::Command;

fn main() {
    let shader_dir = "./shaders";
    let output_dir = "./cshaders";
    println!("Building shaders...");

    // Create output directory if it doesn't exist
    fs::create_dir_all(output_dir).expect("Failed to create output directory");

    // Iterate through files in the shader directory
    for entry in fs::read_dir(shader_dir).expect("Failed to read shader directory") {
        let entry = entry.expect("Failed to read directory entry");
        let path = entry.path();

        if path.is_file() {
            if let Some(extension) = path.extension() {
                if extension == "frag" || extension == "glsl" || extension == "vert" {
                    compile_shader(&path, output_dir);
                }
            }
        }
    }
    println!("Shaders built successfully in {}!", output_dir);
    println!("cargo:rerun-if-changed=shaders");
}

fn compile_shader(shader_path: &Path, output_dir: &str) {
    let file_stem = shader_path.file_stem().unwrap().to_str().unwrap();
    let output_path = format!("{}/{}.spv", output_dir, file_stem);

    let status = Command::new("glslc")
        .arg("-I")
        .arg(shader_path.parent().unwrap())
        .arg(shader_path)
        .arg("-o")
        .arg(&output_path)
        .status()
        .expect("Failed to execute glslc");

    if status.success() {
        println!("Compiled: {} -> {}", shader_path.display(), output_path);
    } else {
        eprintln!("Failed to compile: {}", shader_path.display());
    }
}

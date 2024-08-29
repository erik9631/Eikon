use std::fs;
use std::path::Path;
use std::process::{exit, Command};

fn main() {
    let shader_dir = "./shaders";
    let output_dir = "./cshaders";
    println!("Building shaders...");

    // Create output directory if it doesn't exist
    fs::create_dir_all(output_dir).expect("Failed to create output directory");

    let mut compilation_failed = false;

    // Iterate through files in the shader directory
    for entry in fs::read_dir(shader_dir).expect("Failed to read shader directory") {
        let entry = entry.expect("Failed to read directory entry");
        let path = entry.path();

        if path.is_file() {
            if let Some(extension) = path.extension() {
                if extension == "frag" || extension == "glsl" || extension == "vert" {
                    if !compile_shader(&path, output_dir) {
                        compilation_failed = true;
                    }
                }
            }
        }
    }

    if compilation_failed {
        eprintln!("Shader compilation failed. Build aborted.");
        exit(1);
    } else {
        println!("Shaders built successfully in {}!", output_dir);
    }
    println!("cargo:rerun-if-changed=shaders");
}

fn compile_shader(shader_path: &Path, output_dir: &str) -> bool {
    let file_stem = shader_path.file_stem().unwrap().to_str().unwrap();
    let output_path = format!("{}/{}.spv", output_dir, file_stem);

    let output = Command::new("glslc")
        .arg("-I")
        .arg(shader_path.parent().unwrap())
        .arg(shader_path)
        .arg("-o")
        .arg(&output_path)
        .output()
        .expect("Failed to execute glslc");

    if output.status.success() {
        println!("Compiled: {} -> {}", shader_path.display(), output_path);
        true
    } else {
        eprintln!("Failed to compile: {}", shader_path.display());
        eprintln!("Error output:");
        eprintln!("{}", String::from_utf8_lossy(&output.stderr));
        false
    }
}

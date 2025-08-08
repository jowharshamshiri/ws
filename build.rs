use std::process::Command;
use std::path::Path;

fn main() {
    // Only build Svelte in release mode or when explicitly requested
    let build_svelte = std::env::var("CARGO_CFG_TARGET_ARCH").is_ok() 
        && (std::env::var("PROFILE").unwrap_or_default() == "release" 
            || std::env::var("BUILD_SVELTE").is_ok());
    
    if build_svelte {
        println!("cargo:rerun-if-changed=svelte-dashboard/src");
        println!("cargo:rerun-if-changed=svelte-dashboard/package.json");
        println!("cargo:rerun-if-changed=svelte-dashboard/vite.config.js");
        
        let svelte_dir = Path::new("svelte-dashboard");
        
        if svelte_dir.exists() {
            println!("Building Svelte dashboard...");
            
            // Check if node_modules exists, if not run npm install
            if !svelte_dir.join("node_modules").exists() {
                println!("Installing Svelte dependencies...");
                let install_output = Command::new("npm")
                    .args(&["install"])
                    .current_dir(svelte_dir)
                    .output()
                    .expect("Failed to run npm install");
                
                if !install_output.status.success() {
                    panic!("npm install failed: {}", String::from_utf8_lossy(&install_output.stderr));
                }
            }
            
            // Run the Svelte build
            let build_output = Command::new("npm")
                .args(&["run", "build"])
                .current_dir(svelte_dir)
                .output()
                .expect("Failed to run npm build");
            
            if !build_output.status.success() {
                panic!("Svelte build failed: {}", String::from_utf8_lossy(&build_output.stderr));
            }
            
            println!("Svelte dashboard built successfully!");
        } else {
            println!("Warning: svelte-dashboard directory not found, skipping Svelte build");
        }
    } else {
        println!("Skipping Svelte build (not in release mode)");
    }
}
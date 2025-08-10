use std::env;
use std::path::Path;
use std::process::Command;

fn main() {
    // Only run this script when building in release mode
    if env::var("PROFILE").unwrap() == "release" {
        let out_dir = env::var("OUT_DIR").unwrap();
        let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();

        // Get the target directory (where the executable will be)
        let target_dir = Path::new(&out_dir)
            .ancestors()
            .nth(4) // Go up 4 levels: out -> debug/release -> target -> target_name -> target
            .unwrap()
            .join("release");

        // Create zip file with version (includes HTTP files from project root)
        create_release_zip(&target_dir, &manifest_dir);
    }
}

fn create_release_zip(target_dir: &Path, manifest_dir: &str) {
    // Get the version from Cargo.toml
    let version = env::var("CARGO_PKG_VERSION").unwrap_or_else(|_| "unknown".to_string());
    let package_name = env::var("CARGO_PKG_NAME").unwrap_or_else(|_| "fsr-rs".to_string());

    // Create zip filename with version
    let zip_filename = format!("{}-{}.zip", package_name, version);
    let zip_path = target_dir.join(&zip_filename);

    println!("cargo:warning=Creating release zip: {}", zip_path.display());

    // Determine executable name based on platform
    let exe_name = if cfg!(target_os = "windows") {
        format!("{}.exe", package_name)
    } else {
        package_name.clone()
    };

    let exe_path = target_dir.join(&exe_name);

    if !exe_path.exists() {
        println!(
            "cargo:warning=Executable not found at: {}",
            exe_path.display()
        );
        return;
    }

    // HTTP files source directory (project root)
    let http_source = Path::new(manifest_dir).join("http");

    if !http_source.exists() {
        println!(
            "cargo:warning=HTTP source directory not found at {}",
            http_source.display()
        );
        return;
    }

    // Lua files source directory (project root)
    let lua_source = Path::new(manifest_dir).join("lua");

    if !lua_source.exists() {
        println!(
            "cargo:warning=Lua source directory not found at {}",
            lua_source.display()
        );
        return;
    }

    // Try to use PowerShell's Compress-Archive if available (Windows)
    if cfg!(target_os = "windows") {
        let powershell_result = Command::new("powershell")
            .args(&[
                "-Command",
                &format!(
                    "Compress-Archive -Path '{}', '{}', '{}' -DestinationPath '{}' -Force",
                    exe_path.display(),
                    http_source.display(),
                    lua_source.display(),
                    zip_path.display()
                ),
            ])
            .output();

        if let Ok(output) = powershell_result {
            if output.status.success() {
                println!(
                    "cargo:warning=Release zip created successfully: {}",
                    zip_path.display()
                );
                return;
            } else {
                let error = String::from_utf8_lossy(&output.stderr);
                println!("cargo:warning=PowerShell zip creation failed: {}", error);
            }
        }
    }

    // Fallback: try to use zip command if available
    let zip_result = Command::new("zip")
        .args(&[
            "-r",
            zip_path.to_str().unwrap(),
            exe_name.as_str(),
            http_source.to_str().unwrap(),
            lua_source.to_str().unwrap(),
        ])
        .current_dir(target_dir)
        .output();

    if let Ok(output) = zip_result {
        if output.status.success() {
            println!(
                "cargo:warning=Release zip created successfully: {}",
                zip_path.display()
            );
        } else {
            let error = String::from_utf8_lossy(&output.stderr);
            println!("cargo:warning=Zip creation failed: {}", error);
        }
    } else {
        println!(
            "cargo:warning=No zip tool available. Please install zip or use PowerShell on Windows."
        );
    }
}

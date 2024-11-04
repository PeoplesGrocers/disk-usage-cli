use std::env;
use std::path::Path;
use std::process::Command;

fn main() {
    // Get the OUT_DIR environment variable
    let out_dir = env::var("OUT_DIR").unwrap();

    // Get the project root directory
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let project_root = Path::new(&manifest_dir);

    let esbuild_bin = find_esbuild().expect(
        "\n\nError: esbuild not found!\n\
        Please ensure esbuild is installed by running either:\n\
        \n\
        npm install esbuild    (local install)\n\
        - or -\n\
        npm install -g esbuild (global install)\n\
        \n\
        If you have already installed esbuild, ensure it's available in your PATH\n",
    );

    // Set up paths
    let visualize_dir = project_root.join("src").join("visualize");

    // Prepare esbuild command
    let mut command = Command::new(&esbuild_bin);
    command
        .current_dir(&visualize_dir)
        .arg("--bundle")
        .arg("--external:*.svg")
        .arg("--inject:inject.ts")
        .arg("--loader:.css=local-css")
        .arg("--loader:.html=copy")
        .arg("--mangle-props=_$$")
        .arg("--minify")
        .arg(format!("--outdir={}", out_dir))
        .arg("--target=chrome51")
        .arg("--define:LIVE_RELOAD=false")
        .arg("index.html")
        .arg("index.ts");

    // Run esbuild
    let status = command.status().expect("Failed to execute esbuild");
    if !status.success() {
        panic!("esbuild failed with status: {}", status);
    }

    // Tell Cargo to rerun this build script if any files in src/visualize change
    println!("cargo:rerun-if-changed=src/visualize");
}

fn find_esbuild() -> Option<std::path::PathBuf> {
    // First check if esbuild is in node_modules. I don't use esbuild this way, but if someone is coming into the project cold, and they want to install esbuild locally, then I decided to allow it.
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let project_root = Path::new(&manifest_dir);
    let node_modules = project_root.join("node_modules");
    let local_esbuild = node_modules.join(".bin").join(if cfg!(windows) {
        "esbuild.cmd"
    } else {
        "esbuild"
    });

    if local_esbuild.exists() {
        return Some(local_esbuild);
    }

    if let Ok(output) = Command::new(if cfg!(windows) { "where" } else { "which" })
        .arg("esbuild")
        .output()
    {
        if output.status.success() {
            return Some(String::from_utf8_lossy(&output.stdout).trim().into());
        }
    }

    None
}

use std::process::Command;

fn main() {
    // Client/Core.js
    println!("cargo:rerun-if-changed=src/gen/client/core/src/*/**");
    println!("cargo:rerun-if-changed=src/gen/client/core/package.json");
    Command::new("yarn").current_dir("src/gen/client/core").status().unwrap();
    Command::new("yarn").arg("build").current_dir("src/gen/client/core").status().unwrap();
}

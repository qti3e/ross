use std::process::Command;

fn main() {
    // Client/Core.js
    println!("cargo:rerun-if-changed=src/gen/client/core/core.ts");
    println!("cargo:rerun-if-changed=src/gen/client/core/package.json");
    Command::new("yarn").current_dir("src/gen/client/core").status().unwrap();
    Command::new("npx").arg("tsc").current_dir("src/gen/client/core").status().unwrap();
}

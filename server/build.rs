use std::process::Command;

fn main() {
    // set what version string to use for the build
    // currently it depends on what git outputs, or if failed use "unknown"
    {
        // paths are relative to the workspace root
        println!("cargo:rerun-if-changed=server/build.rs");
        println!("cargo:rerun-if-changed=.git/HEAD");

        // How to read the version:
        // Termusic-server v0.7.11-302-g63396ee5-dirty
        // "Termusic-server" is the binary name
        // "v0.7.11" is the latest tag on the branch
        // "302" is the number of commits since the tag
        // "g63396ee5" is 2 parts, the "g" in the beginning means "git"
        // the rest "63396ee5" is the abbreviated commit sha
        // "dirty" indicates the build has uncommited changes
        let version = Command::new("git")
            .args(["describe", "--tags", "--always", "--dirty"])
            .output()
            .ok()
            .and_then(|v| String::from_utf8(v.stdout).ok())
            .unwrap_or(String::from("unknown"));
        println!("cargo:rustc-env=TERMUSIC_VERSION={version}");
    }
}

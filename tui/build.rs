use std::process::Command;

fn main() {
    // set what version string to use for the build
    // currently it depends on what git outputs, or if failed use "unknown"
    {
        // paths are relative to the workspace root
        println!("cargo:rerun-if-changed=tui/build.rs");
        println!("cargo:rerun-if-changed=.git/HEAD");

        let cargo_toml_version = {
            let version = env!("CARGO_PKG_VERSION");

            if version.is_empty() {
                None
            } else {
                // format is the literal return of what is set in the "Cargo.toml", so to be consistent with git, we modify it to look the same as git (adding "v")
                // also add a inidicator for cargo version
                Some(format!("v{version}[c]"))
            }
        };

        let cargo_toml_or_default = cargo_toml_version.unwrap_or(String::from("unknown"));

        // How to read the git version, first all the variants:
        // Termusic v0.7.11-302-g63396ee5-dirty[g]
        // Termusic v0.7.11-302-g63396ee5[g]
        // Termusic v0.7.11[g]
        // Termusic v0.7.11[c]
        // Termusic unknown
        //
        // "Termusic" is the binary name
        // "v0.7.11" is the latest tag on the branch
        // "302" is the number of commits since the tag, not always present
        // "g63396ee5" is 2 parts, the "g" in the beginning means "git"
        // the rest "63396ee5" is the abbreviated commit sha
        // "dirty" indicates the build has uncommited changes
        // "[g]" at the end means the version has been gotten from "git" ("[c]" means from "cargo")
        let version = Command::new("git")
            .args(["describe", "--tags", "--always", "--dirty"])
            .output()
            .ok()
            .and_then(|v| {
                // print stderr for debugging purposes, it will not be shown unless the build.rs panics
                if !v.stderr.is_empty() {
                    eprintln!(
                        "{}",
                        String::from_utf8(v.stderr).unwrap_or("couldnt parse utf8 stderr".into())
                    );
                }

                String::from_utf8(v.stdout).ok()
            })
            // ignore output if the string gotten is empty
            .and_then(|v| if v.is_empty() { None } else { Some(v) })
            // add a indicator for git version
            .map(|v| format!("{}[g]", v.trim()))
            // fallback to Cargo.toml version, if git is unavailable (like having downloaded the source archive)
            .unwrap_or(cargo_toml_or_default);

        println!("cargo:rustc-env=TERMUSIC_VERSION={version}");
    }
}

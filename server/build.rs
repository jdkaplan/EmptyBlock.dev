use std::env;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    let root = PathBuf::from(&env::var("CARGO_MANIFEST_DIR").unwrap());
    let out = PathBuf::from(&env::var("OUT_DIR").unwrap());

    fs::write(out.join("build_profile"), env::var("PROFILE").unwrap()).unwrap();

    fs::write(out.join("source_url"), env!("CARGO_PKG_REPOSITORY")).unwrap();

    let commit_hash = match git_hash(&root) {
        Some(hash) => hash,
        None => env::var("COMMIT_HASH").unwrap(),
    };

    fs::write(out.join("commit_hash"), commit_hash).unwrap();
}

fn git_hash(repo: &Path) -> Option<String> {
    let hash = {
        let output = Command::new("git")
            .arg("rev-parse")
            .arg("HEAD")
            .current_dir(repo)
            .output()
            .ok()?;

        if output.status.success() {
            String::from_utf8(output.stdout).unwrap().trim().to_string()
        } else {
            return None;
        }
    };

    let suffix = {
        let status = Command::new("git")
            .arg("diff")
            .arg("--quiet")
            .arg("--exit-code")
            .current_dir(repo)
            .status()
            .ok()?;

        if status.success() {
            ""
        } else {
            "+"
        }
    };

    Some(hash + suffix)
}

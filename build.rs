use std::{
    env,
    fmt::Write as _,
    fs::{self, File},
    io::prelude::*,
    path::{Path, PathBuf},
};

// extern crate convert_case
use convert_case::{Case, Casing};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(Debug, Default)]
struct AssetPrint {
    pub asset_root: PathBuf,
    pub buf: String,
    pub indent: usize,
}

impl AssetPrint {
    fn indent(&mut self) {
        for _ in 0..self.indent {
            write!(&mut self.buf, "    ").unwrap();
        }
    }

    fn filter(item: &Path) -> bool {
        let name = item.file_name().unwrap().to_str().unwrap();
        name.starts_with(".")
    }

    pub fn file(&mut self, item: &Path) {
        if Self::filter(item) {
            return;
        }

        self.indent();

        let abs_path = item.canonicalize().unwrap();
        let rel_path = abs_path.strip_prefix(&self.asset_root).unwrap();

        let name = item.file_stem().unwrap().to_str().unwrap();
        let name = name.to_case(Case::UpperSnake);

        writeln!(
            &mut self.buf,
            r#"pub const {}: &str = "{}";"#,
            name,
            rel_path.display()
        )
        .unwrap();
    }

    pub fn push_dir(&mut self, dir: &Path) {
        self.indent();

        let name = dir.components().rev().next().unwrap();
        let name = name.as_os_str().to_str().unwrap();
        let name = name.to_case(Case::Snake);

        writeln!(&mut self.buf, "pub mod {} {{", name,).unwrap();

        self.indent += 1;
    }

    pub fn pop_dir(&mut self) {
        self.indent -= 1;
        self.indent();
        writeln!(&mut self.buf, "}}").unwrap();
    }
}

fn main() -> Result<()> {
    let root = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let asset_root = root.join("assets");
    let dst = root.join("src/assets.rs");
    // if you prefer to not commit `assets.rs`:
    // let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap()).join("assets.rs");
    // and do this in your source file:
    // include!(concat!(env!("OUT_DIR"), "/assets.rs"));

    let mut ap = AssetPrint {
        asset_root: asset_root.clone(),
        buf: String::with_capacity(1024 * 10),
        indent: 0,
    };

    writeln!(&mut ap.buf, "//! Automatically generated with `build.rs`")?;
    writeln!(&mut ap.buf, "")?;

    self::rec(&mut ap, &asset_root)?;

    let mut file = File::create(&dst)?;
    file.write_all(ap.buf.as_bytes())?;

    Ok(())
}

fn rec(ap: &mut AssetPrint, dir: &Path) -> Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            ap.push_dir(&path);
            self::rec(ap, &path)?;
            ap.pop_dir();
        } else if path.is_file() {
            ap.file(&path);
        } else {
            // TODO: handle symlink
            eprintln!("symlink found: {}", path.display());
        }
    }

    Ok(())
}

use std::{
    env,
    fmt::Write as _,
    fs,
    path::{Path, PathBuf},
};

// `extern crate convert_case;` or `cargo add -B convert_case` (if `cargo_edit` is installed)
use convert_case::{Case, Casing};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(Debug)]
struct AssetPrint {
    pub asset_root: PathBuf,
    pub headers: Vec<String>,
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
        name.starts_with(".") || name.ends_with("tiled-project") || name.ends_with("tiled-session")
    }

    pub fn doc_string(&mut self, s: &str) {
        writeln!(&mut self.buf, "{}", s).unwrap();
        writeln!(&mut self.buf, "").unwrap();
        self.headers();
    }

    fn headers(&mut self) {
        for i in 0..self.headers.len() {
            self.indent();
            writeln!(&mut self.buf, "{}", self.headers[i]).unwrap();
        }
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
            r#"pub static {}: &'static AssetKey<'static> = &AssetKey::new_const(Cow::Borrowed(as_path("{}")), None);"#,
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

        self.headers();
    }

    pub fn pop_dir(&mut self) {
        self.indent -= 1;
        self.indent();
        writeln!(&mut self.buf, "}}").unwrap();
    }
}

fn main() -> Result<()> {
    println!("cargo:rerun-if-changed=assets");

    let root = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let asset_root = root.join("assets");
    let dst = root.join("crates/grue2d/src/paths.rs");

    // if you prefer to not commit `paths.rs`:
    //    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    //    let dst = out_dir.join("paths.rs");
    // and do this in your source file:
    //     include!(concat!(env!("OUT_DIR"), "/paths.rs"));

    let headers = vec![
        "#![allow(unused)]".to_string(),
        "use snow2d::asset::AssetKey;".to_string(),
        "use std::{borrow::Cow, ffi::OsStr, path::Path};".to_string(),
        r#"const fn as_path(s:&'static str) -> &'static Path {
            unsafe { &*(s as *const str as *const OsStr as *const Path) }
        }"#
        .to_string(),
    ];

    let mut ap = AssetPrint {
        asset_root: asset_root.clone(),
        headers,
        buf: String::with_capacity(1024 * 10),
        indent: 0,
    };

    ap.doc_string("//! Automatically generated with `build.rs`");
    self::rec(&mut ap, &asset_root)?;

    fs::write(&dst, ap.buf.as_bytes())?;

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

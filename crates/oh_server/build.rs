//! Embed a real Vite production build; absent assets are a build error.
use std::{env, fs, path::Path};
fn collect(root: &Path, dir: &Path, out: &mut Vec<(String, String)>) {
    let mut paths = fs::read_dir(dir)
        .unwrap()
        .map(|e| e.unwrap().path())
        .collect::<Vec<_>>();
    paths.sort();
    for path in paths {
        assert!(!path.is_symlink(), "client dist must not contain symlinks");
        if path.is_dir() {
            collect(root, &path, out);
        } else {
            let relative = path
                .strip_prefix(root)
                .unwrap()
                .to_str()
                .unwrap()
                .replace('\\', "/");
            let mime = match path.extension().and_then(|s| s.to_str()) {
                Some("html") => "text/html; charset=utf-8",
                Some("js") => "text/javascript; charset=utf-8",
                Some("css") => "text/css; charset=utf-8",
                Some("json") => "application/json; charset=utf-8",
                _ => "application/octet-stream",
            };
            out.push((
                format!("/{relative}"),
                format!(
                    "({mime:?}, include_bytes!({:?}) as &'static [u8])",
                    path.canonicalize().unwrap().to_str().unwrap()
                ),
            ));
        }
    }
}
fn main() {
    let dist = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../client/dist");
    println!("cargo:rerun-if-changed={}", dist.display());
    assert!(
        dist.join("index.html").is_file() && dist.join("assets").is_dir(),
        "built client missing: run npm --prefix client ci && npm --prefix client run build BEFORE cargo build/test -p oh_server"
    );
    let mut files = Vec::new();
    collect(&dist, &dist, &mut files);
    assert!(
        files.iter().any(|(path, _)| path.ends_with(".js")),
        "built client has no JavaScript bundle"
    );
    let mut source = String::from(
        "pub fn asset(path: &str) -> Option<(&'static str, &'static [u8])> { match path {\n",
    );
    for (path, value) in files {
        source.push_str(&format!("{path:?} => Some({value}),\n"));
    }
    source.push_str("_ => None } }\n");
    fs::write(
        Path::new(&env::var("OUT_DIR").unwrap()).join("assets.rs"),
        source,
    )
    .unwrap();
}

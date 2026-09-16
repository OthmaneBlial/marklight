use marklight_core::load_file;
use marklight_render::{HtmlOptions, render_html};
use std::{fs, path::Path};

fn main() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../");
    let destination = root.join("artifacts/frontend-fixtures");
    fs::create_dir_all(&destination).unwrap();
    for entry in fs::read_dir(root.join("fixtures/markdown")).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().is_none_or(|e| e != "md") {
            continue;
        }
        let (path, doc) = load_file(&path).unwrap();
        let json = serde_json::json!({"path":path,"name":path.file_name().unwrap().to_string_lossy(),"html":render_html(&doc,&HtmlOptions::default()),"html_dark":render_html(&doc,&HtmlOptions {dark:true,..Default::default()}),"headings":doc.headings,"code_blocks":doc.code_blocks,"metadata":doc.metadata,"recent":[],"warning":null});
        fs::write(
            destination
                .join(path.file_stem().unwrap())
                .with_extension("json"),
            serde_json::to_vec(&json).unwrap(),
        )
        .unwrap();
    }
}

//! End-to-end build test: a temp content dir with frontmatter, links, an image,
//! and a draft; assert the generated output and incremental rebuild.

use std::fs;
use std::path::Path;

use mindmirror::builder::Builder;
use mindmirror::config::Config;
use mindmirror::source::ContentSource;

fn write(path: &Path, contents: &str) {
    if let Some(p) = path.parent() {
        fs::create_dir_all(p).unwrap();
    }
    fs::write(path, contents).unwrap();
}

fn fixture() -> (tempfile::TempDir, Config, ContentSource) {
    let tmp = tempfile::tempdir().unwrap();
    let content = tmp.path().join("content");
    let dest = tmp.path().join("out");

    // Plain page with a markdown link that must be rewritten.
    write(
        &content.join("index.md"),
        "# Home\n\nSee [other](other.md) and [sub](dir/nested.md).\n",
    );
    // Frontmatter-driven page: wide layout, custom style, explicit title.
    write(
        &content.join("other.md"),
        "+++\ntitle = \"Other Page\"\nlayout = \"wide\"\nstyle = \"tufte.css\"\ntags = [\"x\"]\n+++\n\n## Other\n\n$E = mc^2$\n",
    );
    // Nested page.
    write(&content.join("dir/nested.md"), "# Nested\n");
    // Draft must be skipped.
    write(
        &content.join("draft.md"),
        "+++\ndraft = true\n+++\n\n# Secret\n",
    );
    // An image must be copied verbatim.
    write(&content.join("pic.png"), "PNGDATA");

    let mut config = Config::default();
    config.output.dest = dest;
    let source = ContentSource::local("notes", &content).unwrap();
    (tmp, config, source)
}

#[test]
fn builds_site_with_frontmatter_links_images_and_drafts() {
    let (_tmp, config, source) = fixture();
    let builder = Builder::new(&config).unwrap();
    let report = builder.build_source(&source).unwrap();

    let out = config.output.dest.join("notes");

    // Generated counts: index, other, nested = 3 pages; pic.png = 1 image; 1 draft.
    assert_eq!(report.generated, 3, "report: {report:?}");
    assert_eq!(report.images, 1);
    assert_eq!(report.drafts, 1);

    // Link rewriting in index.html.
    let index = fs::read_to_string(out.join("index.html")).unwrap();
    assert!(index.contains("href=\"other.html\""), "{index}");
    assert!(index.contains("href=\"dir/nested.html\""), "{index}");

    // Frontmatter applied: title, wide layout, custom style, math delimiters.
    let other = fs::read_to_string(out.join("other.html")).unwrap();
    assert!(other.contains("<title>Other Page</title>"));
    assert!(other.contains("mm-content-wide"));
    assert!(other.contains("/styles/tufte.css"));
    assert!(other.contains("\\(E = mc^2\\)"));

    // Draft skipped, image copied verbatim.
    assert!(!out.join("draft.html").exists());
    assert_eq!(fs::read_to_string(out.join("pic.png")).unwrap(), "PNGDATA");

    // pages.json excludes the draft and is sorted.
    let pages: Vec<serde_json::Value> =
        serde_json::from_slice(&fs::read(out.join("pages.json")).unwrap()).unwrap();
    let urls: Vec<&str> = pages.iter().map(|p| p["url"].as_str().unwrap()).collect();
    assert_eq!(urls, vec!["dir/nested.html", "index.html", "other.html"]);
}

#[test]
fn incremental_rebuild_skips_unchanged() {
    let (_tmp, config, source) = fixture();
    let builder = Builder::new(&config).unwrap();

    let first = builder.build_source(&source).unwrap();
    assert_eq!(first.generated, 3);
    assert_eq!(first.skipped, 0);

    // No changes → everything skipped, pages still reported.
    let second = builder.build_source(&source).unwrap();
    assert_eq!(second.generated, 0, "report: {second:?}");
    assert_eq!(second.skipped, 4); // 3 md + 1 image
    assert_eq!(second.pages.len(), 3);
}

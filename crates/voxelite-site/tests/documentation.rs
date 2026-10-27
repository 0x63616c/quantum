//! Validate generated documentation and its downloadable project.

fn generated_site() -> tempfile::TempDir {
    let output = tempfile::tempdir().unwrap();
    let result = std::process::Command::new(env!("CARGO_BIN_EXE_voxelite-site"))
        .arg(output.path())
        .current_dir(output.path())
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    output
}

#[test]
fn generated_docs_have_matching_markdown_exports_and_versioned_catalog() {
    let output = generated_site();
    let read = |path: &str| std::fs::read_to_string(output.path().join(path)).unwrap();
    let index = read("llms.txt");
    let full = read("llms-full.txt");
    for slug in ["", "getting-started", "mcp", "format", "skills", "plugins"] {
        let path = if slug.is_empty() {
            "docs".to_owned()
        } else {
            format!("docs/{slug}")
        };
        let markdown = read(&format!("{path}.md"));
        assert!(markdown.starts_with("# "));
        assert_eq!(markdown, read(&format!("{path}/index.md")));
        assert!(read(&format!("{path}/index.html")).contains(&format!(
            "rel=\"alternate\" type=\"text/markdown\" href=\"/{path}.md\""
        )));
        let url = format!("https://voxelite.worldwidewebb.co/{path}.md");
        assert!(index.contains(&format!("]({url}): ")));
        assert!(full.contains(&format!("Source: [{url}]({url})\n\n{markdown}")));
    }
    assert_eq!(
        read("assets/mcp-tools.json"),
        include_str!("../../../spec/mcp/v1/tools.json")
    );
}

#[test]
fn plugins_are_discoverable_in_navigation_and_machine_readable_docs() {
    let output = generated_site();
    let read = |path: &str| std::fs::read_to_string(output.path().join(path)).unwrap();
    for path in ["index.html", "download/index.html", "docs/index.html"] {
        let html = read(path);
        let navigation = html
            .split("<nav aria-label=\"Main navigation\">")
            .nth(1)
            .unwrap()
            .split("</nav>")
            .next()
            .unwrap();
        assert!(
            navigation.contains("href=\"/docs/plugins/\""),
            "Missing Plugins navigation in {path}"
        );
    }
    assert!(read("docs/index.html").contains("<b>Plugins</b>"));
    for slug in ["getting-started", "mcp", "format", "skills", "plugins"] {
        let html = read(&format!("docs/{slug}/index.html"));
        let navigation = html
            .split("<nav class=\"docs-nav\" aria-label=\"Documentation\">")
            .nth(1)
            .unwrap()
            .split("</nav>")
            .next()
            .unwrap();
        assert!(navigation.contains("href=\"/docs/plugins/\""));
        if slug == "plugins" {
            assert!(navigation.contains("href=\"/docs/plugins/\" aria-current=\"page\""));
        }
    }
    let markdown = read("docs/plugins.md");
    assert!(markdown.starts_with("# Plugins\n\n"));
    assert_eq!(markdown, read("docs/plugins/index.md"));
    assert!(read("docs.md").contains("https://voxelite.worldwidewebb.co/docs/plugins.md"));
    assert!(markdown.contains("https://voxelite.worldwidewebb.co/docs/format/#plugin-data"));
    assert!(
        read("docs/format.md").contains("https://voxelite.worldwidewebb.co/docs/plugins/#data")
    );
    let url = "https://voxelite.worldwidewebb.co/docs/plugins";
    assert!(read("llms.txt").contains(&format!("- [Plugins]({url}.md): ")));
    assert!(read("llms-full.txt").contains(&format!("Source: [{url}.md]({url}.md)\n\n{markdown}")));
    for suffix in ["/", ".md"] {
        assert!(read("sitemap.xml").contains(&format!("<loc>{url}{suffix}</loc>")));
    }
}

#[test]
fn downloadable_project_is_valid_and_canonical() {
    let bytes = include_bytes!("../../../site/assets/minimal.voxl");
    let project = voxelite_format::from_bytes(bytes).expect("download must open");
    assert_eq!(project.to_bytes().unwrap(), bytes);
}

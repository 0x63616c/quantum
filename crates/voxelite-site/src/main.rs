//! Static documentation generator. Discovery schemas come from the versioned MCP
//! catalog, which the server's tests keep in sync with its tools.
use serde_json::Value;
use std::{error::Error, fmt::Write as _, fs, path::Path};

mod install;
mod markdown;
mod seo;

const SITE_BASE_URL: &str = "https://voxelite.worldwidewebb.co";
const GITHUB_LINK: &str = include_str!("../templates/github-link.html");

struct DocPage {
    slug: &'static str,
    title: &'static str,
    description: &'static str,
    content: &'static str,
}

impl DocPage {
    fn path(&self) -> String {
        if self.slug.is_empty() {
            "/docs/".to_owned()
        } else {
            format!("/docs/{}/", self.slug)
        }
    }

    fn markdown_path(&self) -> String {
        format!("{}.md", self.path().trim_end_matches('/'))
    }
}

const PAGES: &[DocPage] = &[
    DocPage {
        slug: "",
        title: "Documentation",
        description: "Learn how to build voxel models with Voxelite, connect an AI agent through MCP, work in the native or web editor, and save portable projects.",
        content: include_str!("../../../site/content/overview.html"),
    },
    DocPage {
        slug: "getting-started",
        title: "Getting started",
        description: "Download Voxelite for Mac or build from source, connect your AI agent through MCP, and create your first voxel model with a guide to editing and exports.",
        content: include_str!("../../../site/content/getting-started.html"),
    },
    DocPage {
        slug: "mcp",
        title: "MCP tools",
        description: "Explore Voxelite's MCP tool reference, including input and output schemas, atomic edits, rendered previews, revision guards, and error recovery.",
        content: include_str!("../../../site/content/mcp.html"),
    },
    DocPage {
        slug: "format",
        title: "The .voxl format",
        description: "Understand Voxelite's .voxl project format: JSON structure, voxel coordinates, palettes, layers, canonical hashes, parser limits, and compatibility.",
        content: include_str!("../../../site/content/format.html"),
    },
    DocPage {
        slug: "skills",
        title: "Agent skills",
        description: "Give your AI agent reusable Voxelite skills for model composition, palette design, atomic edits, and visual review, with ready-to-use downloads.",
        content: include_str!("../../../site/content/skills.html"),
    },
    DocPage {
        slug: "plugins",
        title: "Plugins",
        description: "Explore Voxelite's built-in plugins, the core boundary, manifests, lifecycle, versioned project data, and a walkthrough for writing a plugin.",
        content: include_str!("../../../site/content/plugins.html"),
    },
];

fn escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

/// Index authored second-level headings, using the same anchors as the article.
/// Content is trusted build-time HTML; no runtime parsing or script is needed.
fn section_links(content: &str) -> String {
    let mut links = String::new();
    for heading in content.split("<h2 id=\"").skip(1) {
        let Some((id, rest)) = heading.split_once("\">") else {
            continue;
        };
        let Some((label, _)) = rest.split_once("</h2>") else {
            continue;
        };
        let label = match id {
            "web" => "Web editor and site",
            "canonical" => "Bytes and hashes",
            "portable" => "Comments and versions",
            "hands" => "First edit by hand",
            "first-model" => "First model through MCP",
            _ => label,
        };
        write!(links, "<a href=\"#{}\">{label}</a>", escape(id)).expect("string write");
    }
    links
}

fn layout(slug: &str, title: &str, metadata: &str, content: &str) -> String {
    let (markdown_metadata, markdown_actions) = PAGES
        .iter()
        .find(|page| page.slug == slug)
        .map(|page| {
            let path = page.markdown_path();
            (
                format!("\n<link rel=\"alternate\" type=\"text/markdown\" href=\"{path}\" title=\"Markdown\">\n<link rel=\"describedby\" type=\"text/markdown\" href=\"/llms.txt\">"),
                format!("<div class=\"markdown-actions\"><a href=\"{path}\" type=\"text/markdown\" data-markdown>View .md</a><span role=\"status\" aria-live=\"polite\"></span></div>"),
            )
        })
        .unwrap_or_default();
    let sections = section_links(content);
    let mobile_sections = if sections.is_empty() {
        String::new()
    } else {
        format!(
            "<details name=\"docs-menu\"><summary>On this page</summary><nav aria-label=\"Page sections\">{sections}</nav></details>"
        )
    };
    let desktop_sections = if sections.is_empty() {
        String::new()
    } else {
        format!(
            "<aside class=\"docs-toc\"><p class=\"nav-label\">On this page</p><nav aria-label=\"Page sections\">{sections}</nav></aside>"
        )
    };
    let mut navigation = String::new();
    for page in PAGES {
        let href = page.path();
        let label = page.title;
        let current = if page.slug == slug {
            " aria-current=\"page\""
        } else {
            ""
        };
        write!(navigation, "<a href=\"{href}\"{current}>{label}</a>").expect("string write");
    }
    format!(
        r##"<!doctype html><html lang="en" data-theme="dark"><head>
<meta charset="utf-8"><meta name="viewport" content="width=device-width, initial-scale=1">
{metadata}{markdown_metadata}
<meta name="theme-color" content="#000000"><link rel="stylesheet" href="/assets/site.css"><script src="/assets/theme.js"></script><script src="/assets/site.js" defer></script><script>window.va=window.va||function(){{(window.vaq=window.vaq||[]).push(arguments)}};</script><script defer src="/_vercel/insights/script.js"></script><script>window.si=window.si||function(){{(window.siq=window.siq||[]).push(arguments)}};</script><script defer src="/_vercel/speed-insights/script.js"></script></head><body class="documentation">
<a class="skip" href="#main">Skip to content</a><header class="header"><div class="shell nav"><a class="brand" href="/"><svg viewBox="0 0 24 24" aria-hidden="true"><path d="m12 2 9 5v10l-9 5-9-5V7Zm0 10 9-5M12 12 3 7m9 5v10"/></svg>Voxelite</a><nav aria-label="Main navigation"><a href="/">Overview</a><a href="/docs/" aria-current="page">Docs</a><a href="/docs/plugins/">Plugins</a><a href="/download/">Download</a></nav><div class="nav-end"><button class="theme-toggle" type="button" aria-label="Switch to light theme"><svg viewBox="0 0 24 24" aria-hidden="true"><circle cx="12" cy="12" r="4"/><path d="M12 2v2m0 16v2M2 12h2m16 0h2M5 5l1.4 1.4m11.2 11.2L19 19M5 19l1.4-1.4M17.6 6.4 19 5"/></svg></button><a class="button small" href="/app/">Open editor</a></div></div></header>
<div class="shell docs-mobile-nav"><details name="docs-menu"><summary>Browse docs</summary><nav aria-label="Documentation">{navigation}</nav></details>{mobile_sections}</div><div class="shell docs-layout"><nav class="docs-nav" aria-label="Documentation"><p class="nav-label">Voxelite documentation</p>{navigation}</nav><main class="docs-content" id="main"><h1>{title}</h1>{markdown_actions}<p class="build-note">This development build uses API <code>voxelite.v1</code>.</p>{content}<div class="doc-next"><a href="/docs/">All documentation</a> <a href="/docs/getting-started/">Start building</a></div></main>{desktop_sections}</div><footer class="shell footer"><div><a class="brand" href="/">Voxelite</a><span class="caption">by World Wide Webb</span></div><nav aria-label="Footer"><a href="/docs/skills/">Agent skills</a><a href="/docs/plugins/">Plugins</a><a href="/assets/LICENSE">MIT License</a>{github_link}</nav></footer></body></html>
"##,
        github_link = GITHUB_LINK.trim(),
    )
}

fn catalog(tools: &[Value]) -> Result<String, Box<dyn Error>> {
    let mut html = format!(
        "<h2 id=\"catalog\">All {} tools</h2><p>Generated directly from <code>VoxeliteServer::tools()</code>. Expand a schema for nested properties, defaults, limits, conditional requirements and the output envelope. A property marked optional may still be required by a schema condition.</p><div class=\"tool-index\">",
        tools.len()
    );
    for tool in tools {
        let name = tool["name"].as_str().ok_or("tool has no name")?;
        write!(
            html,
            "<a href=\"#{name}\">{}</a>",
            escape(name.trim_start_matches("voxelite_"))
        )?;
    }
    html.push_str("</div>");
    for tool in tools {
        let name = tool["name"].as_str().ok_or("tool has no name")?;
        let description = escape(&tool["description"].as_str().unwrap_or("").replace(
            "Unlock in a separate operation",
            "Clear the lock in a separate operation",
        ));
        let required = tool["inputSchema"]["required"].as_array();
        write!(
            html,
            "<section class=\"tool\" aria-labelledby=\"{name}\"><h3 id=\"{name}\">{name}</h3><p>{description}</p><div class=\"table-scroll\"><table><thead><tr><th scope=\"col\">Argument</th><th scope=\"col\">Required</th><th scope=\"col\">Type / constraints</th></tr></thead><tbody>"
        )?;
        if let Some(properties) = tool["inputSchema"]["properties"].as_object() {
            for (key, schema) in properties {
                let required = required
                    .is_some_and(|fields| fields.iter().any(|field| field.as_str() == Some(key)));
                let mut summary = schema["type"].as_str().unwrap_or("See schema").to_owned();
                for constraint in [
                    "default",
                    "enum",
                    "const",
                    "minimum",
                    "maximum",
                    "minItems",
                    "maxItems",
                    "maxLength",
                ] {
                    if let Some(value) = schema.get(constraint) {
                        write!(summary, "; {constraint}: {value}")?;
                    }
                }
                if let Some(description) = schema["description"].as_str() {
                    write!(summary, ". {description}")?;
                }
                write!(
                    html,
                    "<tr><td><code>{}</code></td><td>{}</td><td>{}</td></tr>",
                    escape(key),
                    if required { "Yes" } else { "Optional" },
                    escape(&summary)
                )?;
            }
        }
        html.push_str("</tbody></table></div>");
        for (key, label) in [
            ("inputSchema", "Complete input schema"),
            ("outputSchema", "Complete output schema"),
        ] {
            if let Some(schema) = tool.get(key) {
                write!(
                    html,
                    "<details><summary>{label}</summary><pre><code class=\"language-json\">{}</code></pre></details>",
                    escape(&serde_json::to_string_pretty(schema)?)
                )?;
            }
        }
        html.push_str("</section>");
    }
    Ok(html)
}

/// Preserve a real feed when supplied, including builds into a separate folder.
/// The normal in-place site build must not copy the file onto itself.
fn copy_appcast(source: &Path, root: &Path) -> std::io::Result<()> {
    if !source.try_exists()? {
        return Ok(());
    }
    let destination = root.join("appcast.xml");
    // Canonical paths can differ for hard links to the same file. Copying onto
    // any alias would truncate the real feed before reading it.
    if destination.try_exists()? && same_file::is_same_file(source, &destination)? {
        return Ok(());
    }
    fs::copy(source, destination)?;
    Ok(())
}

fn article_body(page: &DocPage, tools: &[Value]) -> Result<String, Box<dyn Error>> {
    let mut body = page
        .content
        .replace("<!-- install-block -->", &install::block());
    if page.slug == "skills" {
        body = body.replace("<!-- skill-list -->", &install::skill_list()?);
    }
    if page.slug == "mcp" {
        body.push_str(&catalog(tools)?);
    }
    Ok(body)
}

fn main() -> Result<(), Box<dyn Error>> {
    let destination = std::env::args().nth(1).unwrap_or_else(|| "site".into());
    let root = Path::new(&destination);
    let tools: Vec<Value> = serde_json::from_str(include_str!("../../../spec/mcp/v1/tools.json"))?;
    fs::create_dir_all(root.join("assets"))?;
    install::write_integrations(&root.join("assets/integrations"))?;
    if std::env::args().nth(2).as_deref() == Some("--write-integrations") {
        install::write_integrations(
            &Path::new(env!("CARGO_MANIFEST_DIR")).join("../../integrations"),
        )?;
    }
    copy_appcast(
        &Path::new(env!("CARGO_MANIFEST_DIR")).join("../../site/appcast.xml"),
        root,
    )?;
    fs::write(
        root.join("index.html"),
        format!(
            include_str!("../templates/index.html"),
            metadata = seo::metadata("/", seo::HOME_TITLE, seo::HOME_DESCRIPTION),
            structured_data = seo::software_application(),
            github_link = GITHUB_LINK.trim(),
        ),
    )?;
    fs::create_dir_all(root.join("download"))?;
    fs::write(
        root.join("download/index.html"),
        format!(
            include_str!("../templates/download.html"),
            metadata = seo::metadata(
                "/download/",
                "Download Voxelite for Mac: Voxelite",
                "Download Voxelite for Mac, the free voxel editor built for AI agents. Install on Apple silicon with macOS 13 or later and start creating.",
            ),
            github_link = GITHUB_LINK.trim(),
        ),
    )?;
    fs::write(
        root.join("assets/mcp-tools.json"),
        format!("{}\n", serde_json::to_string_pretty(&tools)?),
    )?;
    let mut documents = Vec::new();
    for page in PAGES {
        let directory = root.join("docs").join(page.slug);
        fs::create_dir_all(&directory)?;
        let body = article_body(page, &tools)?;
        let metadata = seo::metadata(
            &page.path(),
            &format!("{}: Voxelite docs", page.title),
            page.description,
        );
        fs::write(
            directory.join("index.html"),
            layout(page.slug, page.title, &metadata, &body),
        )?;
        let markdown = markdown::render(page, &body)?;
        fs::write(
            root.join(page.markdown_path().trim_start_matches('/')),
            &markdown,
        )?;
        fs::write(directory.join("index.md"), &markdown)?;
        documents.push((page, markdown));
    }
    markdown::write_indexes(root, &documents)?;
    fs::write(
        root.join("404.html"),
        layout(
            "404",
            "A little off the grid",
            &seo::metadata(
                "/404.html",
                "Page not found: Voxelite",
                "This page could not be found. Explore Voxelite's documentation to build voxel models, connect an AI agent, or get started in the browser editor.",
            ),
            "<p>This page could not be found. Head back to the <a href=\"/\">Voxelite homepage</a> or explore the <a href=\"/docs/\">documentation</a>.</p>",
        ),
    )?;
    let mut public_paths = vec!["/".to_owned(), "/download/".to_owned(), "/app/".to_owned()];
    public_paths.extend(PAGES.iter().map(DocPage::path));
    public_paths.extend(PAGES.iter().map(DocPage::markdown_path));
    public_paths.extend(["/llms.txt".to_owned(), "/llms-full.txt".to_owned()]);
    seo::write_crawl_files(root, &public_paths)?;
    println!(
        "Generated homepage, download page, {} documentation pages and {} MCP tool references",
        PAGES.len(),
        tools.len()
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_appcast_does_not_create_a_placeholder() {
        let source = tempfile::tempdir().unwrap();
        let output = tempfile::tempdir().unwrap();
        copy_appcast(&source.path().join("appcast.xml"), output.path()).unwrap();
        assert!(!output.path().join("appcast.xml").exists());
    }

    #[test]
    fn supplied_appcast_is_copied_verbatim_and_preserved_in_place() {
        let source = tempfile::tempdir().unwrap();
        let output = tempfile::tempdir().unwrap();
        let feed = source.path().join("appcast.xml");
        // Temporary test bytes only; no release metadata or repository feed.
        let contents = b"<?xml version=\"1.0\"?>\n<rss version=\"2.0\"/>\n";
        fs::write(&feed, contents).unwrap();
        copy_appcast(&feed, output.path()).unwrap();
        assert_eq!(
            fs::read(output.path().join("appcast.xml")).unwrap(),
            contents
        );
        copy_appcast(&feed, &source.path().join(".")).unwrap();
        assert_eq!(fs::read(&feed).unwrap(), contents);
    }

    #[test]
    fn hard_linked_appcast_does_not_truncate_the_source() {
        let source = tempfile::tempdir().unwrap();
        let output = tempfile::tempdir().unwrap();
        let feed = source.path().join("appcast.xml");
        let contents = b"<?xml version=\"1.0\"?>\n<rss version=\"2.0\"/>\n";
        fs::write(&feed, contents).unwrap();
        fs::hard_link(&feed, output.path().join("appcast.xml")).unwrap();

        copy_appcast(&feed, output.path()).unwrap();

        assert_eq!(fs::read(&feed).unwrap(), contents);
        assert_eq!(
            fs::read(output.path().join("appcast.xml")).unwrap(),
            contents
        );
    }

    #[cfg(unix)]
    #[test]
    fn symbolic_appcast_alias_preserves_the_source() {
        let source = tempfile::tempdir().unwrap();
        let output = tempfile::tempdir().unwrap();
        let feed = source.path().join("appcast.xml");
        let contents = b"<?xml version=\"1.0\"?>\n<rss version=\"2.0\"/>\n";
        fs::write(&feed, contents).unwrap();
        std::os::unix::fs::symlink(&feed, output.path().join("appcast.xml")).unwrap();

        copy_appcast(&feed, output.path()).unwrap();

        assert_eq!(fs::read(&feed).unwrap(), contents);
        assert_eq!(
            fs::read(output.path().join("appcast.xml")).unwrap(),
            contents
        );
    }
}

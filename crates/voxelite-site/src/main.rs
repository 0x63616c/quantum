//! Static documentation generator. Discovery schemas come directly from the MCP
//! server, so API names, required arguments and constraints cannot drift by hand.
use serde_json::Value;
use std::{error::Error, fmt::Write as _, fs, path::Path};
use voxelite_mcp::VoxeliteServer;

const PAGES: &[(&str, &str, &str)] = &[
    (
        "",
        "Documentation",
        include_str!("../../../site/content/overview.html"),
    ),
    (
        "getting-started",
        "Getting started",
        include_str!("../../../site/content/getting-started.html"),
    ),
    (
        "mcp",
        "MCP tools",
        include_str!("../../../site/content/mcp.html"),
    ),
    (
        "format",
        "The .voxl format",
        include_str!("../../../site/content/format.html"),
    ),
    (
        "skills",
        "Agent skills",
        include_str!("../../../site/content/skills.html"),
    ),
];

fn escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

fn layout(slug: &str, title: &str, content: &str) -> String {
    let mut navigation = String::new();
    for (path, label, _) in PAGES {
        let href = if path.is_empty() {
            "/docs/".to_owned()
        } else {
            format!("/docs/{path}/")
        };
        let current = if *path == slug {
            " aria-current=\"page\""
        } else {
            ""
        };
        write!(navigation, "<a href=\"{href}\"{current}>{label}</a>").expect("string write");
    }
    format!(
        r##"<!doctype html><html lang="en" data-theme="dark"><head>
<meta charset="utf-8"><meta name="viewport" content="width=device-width, initial-scale=1">
<title>{title} — Voxelite</title><meta name="description" content="Voxelite documentation: {title}. Build, connect, create and inspect with the Rust voxel editor.">
<meta name="theme-color" content="#000000"><link rel="icon" href="/assets/mark.svg" type="image/svg+xml"><link rel="stylesheet" href="/assets/site.css"><script src="/assets/theme.js"></script><script src="/assets/site.js" defer></script></head><body>
<a class="skip" href="#main">Skip to content</a><header class="header"><div class="shell nav"><a class="brand" href="/"><svg viewBox="0 0 24 24" aria-hidden="true"><path d="m12 2 9 5v10l-9 5-9-5V7Zm0 10 9-5M12 12 3 7m9 5v10"/></svg>Voxelite</a><nav aria-label="Main navigation"><a href="/">Overview</a><a href="/docs/" aria-current="page">Docs</a></nav><div class="nav-end"><button class="theme-toggle" type="button" aria-label="Switch to light theme"><svg viewBox="0 0 24 24" aria-hidden="true"><circle cx="12" cy="12" r="4"/><path d="M12 2v2m0 16v2M2 12h2m16 0h2M5 5l1.4 1.4m11.2 11.2L19 19M5 19l1.4-1.4M17.6 6.4 19 5"/></svg></button><a class="button small" href="/app/">Open editor ↗</a></div></div></header>
<div class="shell docs-layout"><nav class="docs-nav" aria-label="Documentation"><p class="eyebrow">VOXELITE / DOCUMENTATION</p>{navigation}</nav><main class="docs-content" id="main"><p class="eyebrow">DEVELOPMENT BUILD · API VOXELITE.V1</p><h1>{title}</h1>{content}<div class="doc-next"><a href="/docs/">All documentation</a> · <a href="/docs/getting-started/">Start building →</a></div></main></div><footer class="shell footer"><a class="brand" href="/">Voxelite</a><nav aria-label="Footer"><a href="/docs/skills/">Agent skills</a><a href="/assets/LICENSE">MIT License</a></nav><span class="caption">Open source. Local first.</span></footer></body></html>
"##
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
        let description = escape(tool["description"].as_str().unwrap_or(""));
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
                    "<details><summary>{label}</summary><pre><code>{}</code></pre></details>",
                    escape(&serde_json::to_string_pretty(schema)?)
                )?;
            }
        }
        html.push_str("</section>");
    }
    Ok(html)
}

fn main() -> Result<(), Box<dyn Error>> {
    let destination = std::env::args().nth(1).unwrap_or_else(|| "site".into());
    let root = Path::new(&destination);
    let tools: Vec<Value> = VoxeliteServer::tools()
        .into_iter()
        .map(serde_json::to_value)
        .collect::<Result<_, _>>()?;
    fs::create_dir_all(root.join("assets"))?;
    fs::write(
        root.join("assets/mcp-tools.json"),
        format!("{}\n", serde_json::to_string_pretty(&tools)?),
    )?;
    for (slug, title, content) in PAGES {
        let directory = root.join("docs").join(slug);
        fs::create_dir_all(&directory)?;
        let body = if *slug == "mcp" {
            format!("{content}{}", catalog(&tools)?)
        } else {
            (*content).to_owned()
        };
        fs::write(directory.join("index.html"), layout(slug, title, &body))?;
    }
    fs::write(
        root.join("404.html"),
        layout(
            "404",
            "A little off the grid",
            "<p>This page could not be found. Head back to the <a href=\"/\">Voxelite homepage</a> or explore the <a href=\"/docs/\">documentation</a>.</p>",
        ),
    )?;
    println!(
        "Generated {} documentation pages and {} MCP tool references",
        PAGES.len(),
        tools.len()
    );
    Ok(())
}

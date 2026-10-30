//! Shared metadata and crawl files for the static site.
use serde_json::json;
use std::{error::Error, fmt::Write as _, fs, path::Path};

use crate::escape;

/// Change this origin, then regenerate the site. Do not include a trailing slash.
pub const BASE_URL: &str = "https://voxelite.vercel.app";

const IMAGE_PATH: &str = "/assets/session-dark.png";
const HOME_TITLE: &str = "Voxelite — The voxel editor built for AI agents";
const HOME_DESCRIPTION: &str = "Voxelite is a free, open-source Rust voxel editor. Build models through MCP, watch agent edits in the native app, and refine every detail by hand.";
const SEO_START: &str = "<!-- Generated SEO: start -->";
const SEO_END: &str = "<!-- Generated SEO: end -->";

pub fn metadata(path: &str, title: &str, description: &str) -> String {
    let title = escape(title);
    let description = escape(description);
    let url = escape(&format!("{BASE_URL}{path}"));
    let image = escape(&format!("{BASE_URL}{IMAGE_PATH}"));
    let mut html = format!(
        r#"<title>{title}</title>
<meta name="description" content="{description}">
<link rel="canonical" href="{url}">
<meta property="og:title" content="{title}">
<meta property="og:description" content="{description}">
<meta property="og:image" content="{image}">
<meta property="og:url" content="{url}">
<meta property="og:type" content="website">
<meta name="twitter:card" content="summary_large_image">
<meta name="twitter:title" content="{title}">
<meta name="twitter:description" content="{description}">
<meta name="twitter:image" content="{image}">"#
    );
    if path == "/404.html" {
        html.push_str("\n<meta name=\"robots\" content=\"noindex, follow\">");
    }
    if path == "/" {
        let application = json!({
            "@context": "https://schema.org",
            "@type": "SoftwareApplication",
            "name": "Voxelite",
            "description": HOME_DESCRIPTION,
            "url": format!("{BASE_URL}/"),
            "image": format!("{BASE_URL}{IMAGE_PATH}"),
            "applicationCategory": "DesignApplication",
            "operatingSystem": "Linux, Web",
            "isAccessibleForFree": true,
            "license": format!("{BASE_URL}/assets/LICENSE"),
            "offers": {
                "@type": "Offer",
                "price": 0,
                "priceCurrency": "USD"
            }
        });
        // JSON inside a script is raw text, not HTML-escaped attribute content.
        let data = application
            .to_string()
            .replace('<', "\\u003c")
            .replace('>', "\\u003e")
            .replace('&', "\\u0026");
        write!(
            html,
            "\n<script type=\"application/ld+json\">{data}</script>"
        )
        .expect("string write");
    }
    html
}

/// Replace only the metadata block, preserving authored styles, scripts and body.
fn with_metadata(template: &str, metadata: &str) -> Result<String, Box<dyn Error>> {
    let (before, rest) = template
        .split_once(SEO_START)
        .ok_or("missing SEO start marker")?;
    let (_, after) = rest.split_once(SEO_END).ok_or("missing SEO end marker")?;
    Ok(format!("{before}{SEO_START}\n{metadata}\n{SEO_END}{after}"))
}

pub fn write_authored_pages(root: &Path) -> Result<(), Box<dyn Error>> {
    fs::write(
        root.join("index.html"),
        with_metadata(
            include_str!("../../../site/index.html"),
            &metadata("/", HOME_TITLE, HOME_DESCRIPTION),
        )?,
    )?;
    fs::create_dir_all(root.join("app"))?;
    fs::write(
        root.join("app/index.html"),
        with_metadata(
            include_str!("../../../web/index.html"),
            &metadata(
                "/app/",
                "Voxelite",
                "Create and edit voxel models in the Voxelite browser editor. Shape the Autumn Workshop, choose palettes, and save your work as local project files.",
            ),
        )?,
    )?;
    Ok(())
}

pub fn write_crawl_files(root: &Path, paths: &[String]) -> Result<(), Box<dyn Error>> {
    let mut sitemap = String::from(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<urlset xmlns=\"http://www.sitemaps.org/schemas/sitemap/0.9\">\n",
    );
    for path in paths {
        writeln!(
            sitemap,
            "  <url><loc>{}</loc></url>",
            escape(&format!("{BASE_URL}{path}"))
        )?;
    }
    sitemap.push_str("</urlset>\n");
    fs::write(root.join("sitemap.xml"), sitemap)?;
    fs::write(
        root.join("robots.txt"),
        format!(
            "User-agent: *\nAllow: /\nDisallow: /content/\n\nSitemap: {BASE_URL}/sitemap.xml\n"
        ),
    )?;
    Ok(())
}

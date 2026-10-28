//! Shared page metadata and crawl files, generated from the production origin.
use serde_json::json;
use std::{error::Error, fmt::Write as _, path::Path};

use crate::{SITE_BASE_URL, escape};

pub const HOME_TITLE: &str = "Quantum: The voxel editor built for AI agents";
pub const HOME_DESCRIPTION: &str = "Quantum is a free Rust voxel editor. Build models through MCP, watch agent edits in the native app, and refine every detail by hand.";
const IMAGE_PATH: &str = "/assets/session-dark.png";
const IMAGE_ALT: &str =
    "Quantum editor showing the Autumn Workshop model, palette, layers, and agent activity.";

pub fn metadata(path: &str, title: &str, description: &str) -> String {
    let title = escape(title);
    let description = escape(description);
    let url = escape(&format!("{SITE_BASE_URL}{path}"));
    let image = escape(&format!("{SITE_BASE_URL}{IMAGE_PATH}"));
    let indexing = if path == "/404.html" {
        "<meta name=\"robots\" content=\"noindex, follow\">".to_owned()
    } else {
        format!("<link rel=\"canonical\" href=\"{url}\">")
    };
    format!(
        r#"<title>{title}</title>
<meta name="description" content="{description}">
{indexing}
<meta property="og:title" content="{title}">
<meta property="og:description" content="{description}">
<meta property="og:url" content="{url}">
<meta property="og:image" content="{image}">
<meta property="og:image:alt" content="{IMAGE_ALT}">
<meta property="og:type" content="website">
<meta property="og:site_name" content="Quantum">
<meta name="twitter:card" content="summary_large_image">
<meta name="twitter:title" content="{title}">
<meta name="twitter:description" content="{description}">
<meta name="twitter:image" content="{image}">
<meta name="twitter:image:alt" content="{IMAGE_ALT}">
<link rel="icon" href="/assets/mark.svg" type="image/svg+xml">
<link rel="preload" href="/assets/fonts/Geist.woff2" as="font" type="font/woff2" crossorigin>
<link rel="preload" href="/assets/fonts/GeistMono.woff2" as="font" type="font/woff2" crossorigin>"#
    )
}

pub fn software_application() -> String {
    let organization = json!({"@type": "Organization", "name": "World Wide Webb"});
    let application = json!({
        "@context": "https://schema.org",
        "@type": "SoftwareApplication",
        "name": "Quantum",
        "description": HOME_DESCRIPTION,
        "url": format!("{SITE_BASE_URL}/"),
        "image": format!("{SITE_BASE_URL}{IMAGE_PATH}"),
        "applicationCategory": "DesignApplication",
        "operatingSystem": "macOS",
        "isAccessibleForFree": true,
        "license": format!("{SITE_BASE_URL}/assets/LICENSE"),
        "offers": {"@type": "Offer", "price": 0, "priceCurrency": "USD"},
        "author": organization,
        "publisher": organization,
    });
    // Script contents are raw text; preserve JSON while preventing HTML parsing.
    let data = application.to_string().replace('<', "\\u003c");
    format!("<script type=\"application/ld+json\">{data}</script>")
}

pub fn write_crawl_files(root: &Path, paths: &[String]) -> Result<(), Box<dyn Error>> {
    let mut sitemap = String::from(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<urlset xmlns=\"http://www.sitemaps.org/schemas/sitemap/0.9\">\n",
    );
    for path in paths {
        writeln!(
            sitemap,
            "  <url><loc>{}</loc></url>",
            escape(&format!("{SITE_BASE_URL}{path}"))
        )?;
    }
    sitemap.push_str("</urlset>\n");
    voxelite_format::storage::safe_write(root.join("sitemap.xml"), sitemap)?;
    voxelite_format::storage::safe_write(
        root.join("robots.txt"),
        format!(
            "User-agent: *\nAllow: /\n\nSitemap: {SITE_BASE_URL}/sitemap.xml\n\n# Markdown documentation: {SITE_BASE_URL}/llms.txt\n# Complete documentation: {SITE_BASE_URL}/llms-full.txt\n"
        ),
    )?;
    Ok(())
}

//! Markdown exports use the same article fragments and live catalog as the HTML.
use htmd::{
    Element, HtmlToMarkdown,
    element_handler::Handlers,
    options::{BrStyle, Options},
};
use std::{error::Error, fmt::Write as _, fs, path::Path};
use url::Url;

use crate::{DocPage, PAGES, SITE_BASE_URL, escape};

pub fn render(page: &DocPage, body: &str) -> Result<String, Box<dyn Error>> {
    let base = Url::parse(&format!("{SITE_BASE_URL}{}", page.path()))?;
    let converter = HtmlToMarkdown::builder()
        .options(Options {
            br_style: BrStyle::Backslash,
            ..Default::default()
        })
        .skip_tags(vec!["script", "style"])
        .add_handler(
            vec!["div", "nav"],
            |handlers: &dyn Handlers, element: Element| {
                let link_list = element.attrs.iter().any(|attr| {
                    attr.name.local.as_ref() == "class"
                        && attr.value.split_whitespace().any(|class| {
                            matches!(class, "docs-links" | "tool-index" | "setup-paths")
                        })
                });
                if !link_list {
                    return handlers.fallback(element);
                }
                let mut list = String::from("\n\n");
                for child in element.node.children.borrow().iter() {
                    if let Some(result) = handlers.handle(child)
                        && !result.content.trim().is_empty()
                    {
                        writeln!(list, "- {}", result.content.trim()).expect("string write");
                    }
                }
                list.push('\n');
                Some(list.into())
            },
        )
        .add_handler(
            vec!["a", "img"],
            move |handlers: &dyn Handlers, element: Element| {
                let mut attrs = element.attrs.to_vec();
                for attr in &mut attrs {
                    if matches!(attr.name.local.as_ref(), "href" | "src")
                        && let Ok(mut url) = base.join(&attr.value)
                    {
                        // Absolute URLs work from either raw alias and from the
                        // concatenated document. Keep authored fragment links on
                        // HTML, where their explicit IDs are guaranteed to exist.
                        if url.origin() == base.origin()
                            && url.fragment().is_none()
                            && let Some(target) = PAGES.iter().find(|page| {
                                url.path() == page.path()
                                    || url.path() == format!("{}index.html", page.path())
                            })
                        {
                            url.set_path(&target.markdown_path());
                        }
                        attr.value = url.as_str().into();
                    }
                }
                let mut element: Element<'_> = element;
                element.attrs = &attrs;
                handlers.fallback(element)
            },
        )
        .build();
    let article = format!("<h1>{}</h1>{body}", escape(page.title));
    Ok(format!("{}\n", converter.convert(&article)?.trim_end()))
}

pub fn write_indexes(root: &Path, documents: &[(&DocPage, String)]) -> Result<(), Box<dyn Error>> {
    let mut index = format!(
        "# Voxelite\n\n> A voxel editor with native Mac and browser workspaces, an MCP interface for AI agents, and portable .voxl projects.\n\n\
         These docs describe the development build. The MCP reference is generated from the live server catalog.\n\n\
         All documentation in one file: [llms-full.txt]({SITE_BASE_URL}/llms-full.txt).\n\n## Documentation\n\n"
    );
    let mut full = String::from(
        "# Voxelite documentation\n\n> Complete documentation, generated from the same source content and MCP catalog as the website.\n",
    );
    for (page, markdown) in documents {
        let url = format!("{SITE_BASE_URL}{}", page.markdown_path());
        writeln!(index, "- [{}]({url}): {}", page.title, page.description)?;
        write!(full, "\n---\n\nSource: [{url}]({url})\n\n{markdown}")?;
    }
    fs::write(root.join("llms.txt"), index)?;
    fs::write(root.join("llms-full.txt"), full)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exports_semantics_and_links_without_page_chrome() {
        let body = r##"<h2 id="example">Example &amp; details</h2>
<p><a href="../format/">Format</a> <a href="#example">This section</a>
<a href="/docs/getting-started/#connect">Connect</a></p>
<table><tr><th>Field</th><th>Value</th></tr><tr><td>name</td><td>a|b</td></tr></table>
<details><summary>Complete schema</summary><pre><code class="language-json">{"value": "&lt;returned id&gt;", "lines": [1, 2]}</code></pre></details>
<ol><li>First</li><li>Second<ul><li>Nested</li></ul></li></ol>
<blockquote><p>Inspect the model.</p></blockquote><script>ignored()</script>"##;
        let markdown = render(&PAGES[1], body).unwrap();
        assert!(markdown.starts_with("# Getting started\n"));
        assert!(markdown.contains("## Example & details"));
        assert!(markdown.contains(&format!("[Format]({SITE_BASE_URL}/docs/format.md)")));
        assert!(markdown.contains(&format!(
            "[This section]({SITE_BASE_URL}/docs/getting-started/#example)"
        )));
        assert!(markdown.contains(&format!(
            "[Connect]({SITE_BASE_URL}/docs/getting-started/#connect)"
        )));
        assert!(markdown.contains("| Field | Value |"));
        assert!(markdown.contains("a&#124;b"));
        assert!(markdown.contains("Complete schema\n"));
        assert!(
            markdown.contains("```json\n{\"value\": \"<returned id>\", \"lines\": [1, 2]}\n```")
        );
        assert!(markdown.contains("1.  First\n2.  Second"));
        assert!(markdown.contains("Nested"));
        assert!(markdown.contains("> Inspect the model."));
        assert!(!markdown.contains("ignored()"));
    }
}

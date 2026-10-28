//! Release history and Markdown blog articles share the existing page shell.
use crate::{brand, escape, layout, seo};
use pulldown_cmark::{Options, Parser, html};
use std::{error::Error, fmt::Write as _, fs, path::Path};

fn markdown(source: &str) -> String {
    let mut output = String::new();
    html::push_html(&mut output, Parser::new_ext(source, Options::ENABLE_TABLES));
    output
}

fn page(root: &Path, path: &str, title: &str, source: &str) -> Result<(), Box<dyn Error>> {
    let directory = root.join(path.trim_matches('/'));
    fs::create_dir_all(&directory)?;
    let source = brand::expand(source);
    let description = format!(
        "{title}. Updates from @@product@@, the voxel editor for people and AI agents. Read project news and explore the latest changes to the editor."
    );
    let metadata = seo::metadata(path, &format!("{title}: @@product@@"), &description);
    brand::write(
        directory.join("index.html"),
        layout(path, &escape(title), &metadata, &markdown(&source)),
    )?;
    brand::write(directory.join("index.md"), format!("# {title}\n\n{source}"))?;
    Ok(())
}

pub fn write(root: &Path) -> Result<Vec<String>, Box<dyn Error>> {
    let repository = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let changelog = fs::read_to_string(repository.join("CHANGELOG.md"))?;
    let changelog = changelog.replace(
        "(docs/",
        &format!(
            "({}/blob/main/docs/",
            brand::get().identifiers.github_repo_url.current
        ),
    );
    let (_, releases) = changelog
        .split_once('\n')
        .ok_or("missing changelog heading")?;
    page(root, "/changelog/", "Changelog", releases)?;
    let mut paths = vec!["/changelog/".into(), "/blog/".into()];
    let mut posts = fs::read_dir(repository.join("site/content/blog"))?
        .map(|entry| entry.map(|entry| entry.path()))
        .collect::<Result<Vec<_>, _>>()?;
    posts.sort();
    posts.reverse();
    let mut index = String::from("<p>News from the editor.</p><ul>");
    for post in posts {
        if post.extension().and_then(|value| value.to_str()) != Some("md") {
            continue;
        }
        let slug = post
            .file_stem()
            .and_then(|value| value.to_str())
            .ok_or("invalid blog filename")?;
        if !slug
            .bytes()
            .all(|value| value.is_ascii_lowercase() || value.is_ascii_digit() || value == b'-')
        {
            return Err("blog filenames must be lowercase URL slugs".into());
        }
        let source = fs::read_to_string(&post)?;
        let (title, body) = source.split_once('\n').ok_or("missing blog title")?;
        let title = brand::expand(
            title
                .strip_prefix("# ")
                .ok_or("blog must start with # title")?,
        );
        let path = format!("/blog/{slug}/");
        page(root, &path, &title, body)?;
        write!(index, "<li><a href=\"{path}\">{}</a></li>", escape(&title))?;
        paths.push(path);
    }
    index.push_str("</ul>");
    page(root, "/blog/", "Blog", &index)?;
    Ok(paths)
}

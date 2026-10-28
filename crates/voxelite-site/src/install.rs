//! Every host installs the same stdio server; only the root syntax varies.
use base64::{Engine as _, engine::general_purpose::STANDARD};
use percent_encoding::{AsciiSet, NON_ALPHANUMERIC, utf8_percent_encode};
use serde::Serialize;
use serde_json::{Value, json};
use std::{error::Error, fmt::Write as _, fs, path::Path};

use crate::escape;

const NAME: &str = "voxelite";
const SKILLS_COMMAND: &str = "npx skills add 0x63616c/voxelite";
// encodeURIComponent's unescaped characters, including punctuation in paths.
const COMPONENT: &AsciiSet = &NON_ALPHANUMERIC
    .remove(b'-')
    .remove(b'_')
    .remove(b'.')
    .remove(b'~')
    .remove(b'!')
    .remove(b'\'')
    .remove(b'(')
    .remove(b')')
    .remove(b'*');

#[derive(Clone, Copy, Serialize)]
struct ServerConfig<'a> {
    command: &'a str,
    args: [&'a str; 2],
}

#[derive(Serialize)]
struct NamedServerConfig<'a> {
    name: &'static str,
    #[serde(flatten)]
    server: ServerConfig<'a>,
}

const SERVER: ServerConfig<'_> = ServerConfig {
    command: "/Applications/Voxelite.app/Contents/MacOS/voxelite-mcp",
    args: ["--root", "${workspaceFolder}"],
};

impl<'a> ServerConfig<'a> {
    fn with_root(self, root: &'a str) -> Self {
        Self {
            args: [self.args[0], root],
            ..self
        }
    }

    #[expect(clippy::expect_used, reason = "internal invariant: server JSON")]
    fn named_json(self) -> String {
        serde_json::to_string(&NamedServerConfig {
            name: NAME,
            server: self,
        })
        .expect("server JSON")
    }

    fn generic(self) -> Value {
        json!({"mcpServers": {NAME: self}})
    }

    fn cli(self, host: &str) -> String {
        format!(
            "{host} mcp add {NAME} -- {} {} \"$PWD\"",
            shell_quote(self.command),
            self.args[0]
        )
    }

    fn toml(self) -> String {
        format!(
            "[mcp_servers.{NAME}]\ncommand = {}\nargs = {}",
            json!(self.command),
            json!(self.args)
        )
    }
}

fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}

#[expect(clippy::expect_used, reason = "internal invariant: server JSON")]
fn cursor_link() -> String {
    let encoded = STANDARD.encode(serde_json::to_string(&SERVER).expect("server JSON"));
    format!("cursor://anysphere.cursor-deeplink/mcp/install?name={NAME}&config={encoded}")
}

fn vscode_link() -> String {
    format!(
        "vscode:mcp/install?{}",
        utf8_percent_encode(&SERVER.named_json(), COMPONENT)
    )
}

fn vscode_cli() -> String {
    format!("code --add-mcp {}", shell_quote(&SERVER.named_json()))
}

fn code(language: &str, text: &str) -> String {
    format!(
        "<pre><code class=\"language-{language}\">{}</code></pre>",
        escape(text)
    )
}

#[expect(clippy::expect_used, reason = "internal invariant: server JSON")]
pub fn block() -> String {
    format!(
        include_str!("../templates/install.html"),
        cursor_link = escape(&cursor_link()),
        vscode_link = escape(&vscode_link()),
        claude_command = code("sh", &SERVER.cli("claude")),
        codex_command = code("sh", &SERVER.cli("codex")),
        vscode_command = code("sh", &vscode_cli()),
        codex_config = code("toml", &SERVER.with_root("<project dir>").toml()),
        generic_config = code(
            "json",
            &serde_json::to_string_pretty(&SERVER.with_root("<project dir>").generic())
                .expect("server JSON")
        ),
        skills_command = code("sh", SKILLS_COMMAND),
    )
}

pub fn skill_list() -> Result<String, Box<dyn Error>> {
    let directory = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../skills");
    let mut paths = fs::read_dir(directory)?
        .map(|entry| entry.map(|entry| entry.path()))
        .collect::<Result<Vec<_>, _>>()?;
    paths.sort();
    let mut html = String::from("<h2 id=\"available-skills\">Choose a skill</h2>");
    for path in paths {
        let skill = path.join("SKILL.md");
        if !skill.is_file() {
            continue;
        }
        let source = fs::read_to_string(skill)?;
        let metadata = source
            .strip_prefix("---\n")
            .and_then(|text| text.split_once("\n---"))
            .ok_or("missing skill metadata")?
            .0;
        let field = |name: &str| metadata.lines().find_map(|line| line.strip_prefix(name));
        let name = field("name: ").ok_or("missing skill name")?;
        let description = field("description: ").ok_or("missing skill description")?;
        // Skill names are both CLI arguments and URL path components.
        if !name.bytes().all(|c| c.is_ascii_lowercase() || c == b'-')
            || path.file_name().and_then(|p| p.to_str()) != Some(name)
        {
            return Err("invalid skill directory/name".into());
        }
        write!(
            html,
            "<section><h3 id=\"{name}\">{name}</h3><p>{}</p>{}<p><a href=\"/assets/skills/{name}/SKILL.md\">Read skill instructions</a></p></section>",
            escape(description),
            code("sh", &format!("{SKILLS_COMMAND} --skill {name}"))
        )?;
    }
    Ok(html)
}

/// The build script refreshes the repository examples and downloadable copies.
#[expect(clippy::expect_used, reason = "internal invariant: example directory")]
pub fn write_integrations(root: &Path) -> Result<(), Box<dyn Error>> {
    let manual = SERVER.with_root("<project dir>");
    for (path, config) in [
        ("cursor/mcp.json", SERVER.generic()),
        ("claude/mcp.json", manual.generic()),
        ("other/mcp.json", manual.generic()),
        ("vscode/mcp.json", json!({"servers": {NAME: SERVER}})),
    ] {
        let path = root.join(path);
        fs::create_dir_all(path.parent().expect("example directory"))?;
        voxelite_format::storage::safe_write(
            path,
            format!("{}\n", serde_json::to_string_pretty(&config)?),
        )?;
    }
    fs::create_dir_all(root.join("codex"))?;
    voxelite_format::storage::safe_write(
        root.join("codex/config.toml"),
        format!("{}\n", manual.toml()),
    )?;
    voxelite_format::storage::safe_write(
        root.join("README.md"),
        format!(
            include_str!("../templates/integrations.md"),
            command = SERVER.command,
            cursor_link = cursor_link(),
            vscode_link = vscode_link(),
            claude_command = SERVER.cli("claude"),
            codex_command = SERVER.cli("codex"),
            vscode_command = vscode_cli(),
        ),
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{PAGES, article_body, markdown};
    use percent_encoding::percent_decode_str;
    use url::Url;

    #[test]
    fn every_install_link_and_command_survives_html_and_markdown() {
        let expected = json!({"command": "/Applications/Voxelite.app/Contents/MacOS/voxelite-mcp", "args": ["--root", "${workspaceFolder}"]});
        assert_eq!(serde_json::to_value(SERVER).unwrap(), expected);
        for page in PAGES
            .iter()
            .filter(|page| matches!(page.slug, "mcp" | "skills"))
        {
            let html = article_body(page, &[]).unwrap();
            let markdown = markdown::render(page, &html).unwrap();
            let mut counts = [0, 0];
            for href in html
                .split("href=\"")
                .skip(1)
                .map(|text| text.split('"').next().unwrap().replace("&amp;", "&"))
            {
                if href.starts_with("cursor:") {
                    let url = Url::parse(&href).unwrap();
                    assert_eq!(url.host_str(), Some("anysphere.cursor-deeplink"));
                    assert_eq!(url.path(), "/mcp/install");
                    let query: std::collections::HashMap<_, _> = url.query_pairs().collect();
                    assert_eq!(query["name"], NAME);
                    let bytes = STANDARD.decode(query["config"].as_bytes()).unwrap();
                    assert_eq!(serde_json::from_slice::<Value>(&bytes).unwrap(), expected);
                    assert_eq!(bytes, serde_json::to_vec(&SERVER).unwrap());
                    assert!(markdown.contains(&format!("[Add to Cursor]({href})")));
                    counts[0] += 1;
                } else if let Some(encoded) = href.strip_prefix("vscode:mcp/install?") {
                    let decoded = percent_decode_str(encoded).decode_utf8().unwrap();
                    let value: Value = serde_json::from_str(&decoded).unwrap();
                    assert_eq!(decoded, SERVER.named_json());
                    assert_eq!(value["name"], NAME);
                    assert_eq!(value["command"], expected["command"]);
                    assert_eq!(value["args"], expected["args"]);
                    assert!(markdown.contains(&format!("[Install in VS Code]({href})")));
                    counts[1] += 1;
                }
            }
            assert_eq!(counts, [1, 1]);
            for command in [
                SERVER.cli("claude"),
                SERVER.cli("codex"),
                vscode_cli(),
                SKILLS_COMMAND.to_owned(),
                SERVER.with_root("<project dir>").toml(),
            ] {
                assert!(html.contains(&escape(&command)), "{command}");
                assert!(markdown.contains(&command), "{command}");
            }
            if page.slug == "skills" {
                for name in [
                    "voxelite-workflow",
                    "voxelite-palettes",
                    "voxelite-primitives",
                    "voxelite-render-review",
                ] {
                    assert!(markdown.contains(&format!("{SKILLS_COMMAND} --skill {name}")));
                }
            }
            assert!(
                !markdown.contains("!["),
                "Badges should export as plain links"
            );
        }
    }
}

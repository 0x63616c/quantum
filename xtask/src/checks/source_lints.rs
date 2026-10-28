//! Syntax-aware source budgets. Baselines must exactly match remaining debt.
use std::collections::BTreeMap;

use proc_macro2::{TokenStream, TokenTree};
use quote::ToTokens;
use syn::{
    spanned::Spanned,
    visit::{self, Visit},
};

use crate::{
    Error, Result,
    check::{Context, Outcome},
};

type Counts = BTreeMap<String, usize>;
const FILE_LINES: usize = 1000;
const FUNCTION_LINES: usize = 100;
const ALLOWLIST: &str = "xtask/lint-allowlist.json";

pub(crate) fn run(context: &Context<'_>) -> Outcome {
    super::outcome((|| {
        let mut actual = Counts::new();
        for path in context.workspace.tracked_files(context.runner)? {
            if !path.ends_with(".rs")
                || !(path.starts_with("crates/")
                    || path.starts_with("xtask/")
                    || path.starts_with("examples/"))
            {
                continue;
            }
            let source = context.workspace.read(&path)?;
            inspect(&path, &source, &mut actual)?;
        }
        let baseline: Counts =
            serde_json::from_str(&context.workspace.read(ALLOWLIST)?).map_err(|source| {
                Error::Json {
                    context: ALLOWLIST.into(),
                    source,
                }
            })?;
        let mut errors = ratchet(&actual, &baseline);
        {
            let reference = std::env::var_os("LINT_BASE_REF").unwrap_or_else(|| "HEAD".into());
            let reference = reference
                .to_str()
                .ok_or_else(|| Error::Invalid("LINT_BASE_REF must be UTF-8".into()))?;
            if let Some(previous) = previous_baseline(context, reference)? {
                let previous = follow_renames(context, reference, previous)?;
                errors.extend(no_growth(&baseline, &previous));
            }
        }
        Ok(errors)
    })())
}

// The first rollout has no baseline on its base revision. A missing/invalid
// revision is an error; only an absent baseline file bootstraps the ratchet.
fn previous_baseline(context: &Context<'_>, reference: &str) -> Result<Option<Counts>> {
    let sh = xshell::Shell::new().map_err(|error| Error::Invalid(error.to_string()))?;
    let commit = format!("{reference}^{{commit}}");
    let command = crate::cmd::Command::from(xshell::cmd!(sh, "git rev-parse --verify {commit}"))
        .current_dir(context.workspace.root());
    let commit = command.read(context.runner)?;
    let commit = commit.trim();
    let command = crate::cmd::Command::from(xshell::cmd!(
        sh,
        "git ls-tree --name-only {commit} -- {ALLOWLIST}"
    ))
    .current_dir(context.workspace.root());
    if command.read(context.runner)?.trim().is_empty() {
        return Ok(None);
    }
    let object = format!("{commit}:{ALLOWLIST}");
    let command = crate::cmd::Command::from(xshell::cmd!(sh, "git show {object}"))
        .current_dir(context.workspace.root());
    serde_json::from_str(&command.read(context.runner)?)
        .map(Some)
        .map_err(|source| Error::Json {
            context: object,
            source,
        })
}

// A path move retains its existing budget; Git must recognize a real rename.
// Rule/function keys and counts remain unchanged, so moves cannot grow debt.
fn follow_renames(context: &Context<'_>, reference: &str, previous: Counts) -> Result<Counts> {
    let sh = xshell::Shell::new().map_err(|error| Error::Invalid(error.to_string()))?;
    let command = crate::cmd::Command::from(xshell::cmd!(
        sh,
        "git diff --name-status -z --find-renames --diff-filter=R {reference} --"
    ))
    .current_dir(context.workspace.root());
    remap_counts(previous, &command.read(context.runner)?)
}

fn remap_counts(previous: Counts, diff: &str) -> Result<Counts> {
    let mut renames = BTreeMap::new();
    let mut fields = diff.split_terminator('\0');
    while let Some(status) = fields.next() {
        let (Some(old), Some(new)) = (fields.next(), fields.next()) else {
            return Err(Error::Invalid("Incomplete Git rename record".into()));
        };
        if !status
            .strip_prefix('R')
            .is_some_and(|n| !n.is_empty() && n.bytes().all(|b| b.is_ascii_digit()))
        {
            return Err(Error::Invalid("Expected a Git rename record".into()));
        }
        renames.insert(old, new);
    }
    let mut mapped = Counts::new();
    for (key, count) in previous {
        let key = match key.split_once(':') {
            Some((path, rule)) if renames.contains_key(path) => format!("{}:{rule}", renames[path]),
            _ => key,
        };
        if mapped.insert(key, count).is_some() {
            return Err(Error::Invalid("Rename merges distinct lint budgets".into()));
        }
    }
    Ok(mapped)
}

fn no_growth(current: &Counts, previous: &Counts) -> Vec<String> {
    current
        .iter()
        .filter_map(|(key, count)| {
            let old = previous.get(key).copied().unwrap_or(0);
            (*count > old).then(|| {
                format!("{key}: allowlist grew from {old} to {count}; {ALLOWLIST} may only shrink")
            })
        })
        .collect()
}

fn ratchet(actual: &Counts, baseline: &Counts) -> Vec<String> {
    let mut errors = Vec::new();
    for key in actual
        .keys()
        .chain(baseline.keys())
        .collect::<std::collections::BTreeSet<_>>()
    {
        let observed = actual.get(key).copied().unwrap_or(0);
        let allowed = baseline.get(key).copied().unwrap_or(0);
        if observed > allowed {
            errors.push(format!(
                "{key}: {observed} exceeds allowed {allowed}; refactor the new violation"
            ));
        } else if observed < allowed || allowed == 0 {
            errors.push(format!("{key}: now {observed}, allowlist {allowed}; reduce/remove this entry in {ALLOWLIST}"));
        }
    }
    errors
}

fn inspect(path: &str, source: &str, counts: &mut Counts) -> Result<()> {
    let syntax =
        syn::parse_file(source).map_err(|error| Error::Invalid(format!("{path}: {error}")))?;
    let lines = source.lines().count();
    if lines > FILE_LINES {
        counts.insert(format!("{path}:file-lines"), lines);
    }
    let mut visitor = Inspector {
        path,
        counts,
        scope: Vec::new(),
        app: path.starts_with("crates/quantum-app/src/"),
    };
    if visitor.app {
        visitor.styles(syntax.to_token_stream());
    }
    visitor.visit_file(&syntax);
    Ok(())
}

struct Inspector<'a> {
    path: &'a str,
    counts: &'a mut Counts,
    scope: Vec<String>,
    app: bool,
}
impl Inspector<'_> {
    fn count(&mut self, rule: &str) {
        *self
            .counts
            .entry(format!("{}:{rule}", self.path))
            .or_default() += 1;
    }
    fn styles(&mut self, tokens: TokenStream) {
        let tokens: Vec<_> = tokens.into_iter().collect();
        for pair in tokens.windows(2) {
            if let [TokenTree::Ident(name), TokenTree::Group(args)] = pair {
                if args.delimiter() != proc_macro2::Delimiter::Parenthesis {
                    continue;
                }
                let name = name.to_string();
                let args = args.stream();
                // Parse expressions to distinguish literals from token references.
                if let Ok(args) = syn::parse::Parser::parse2(
                    syn::punctuated::Punctuated::<syn::Expr, syn::Token![,]>::parse_terminated,
                    args,
                ) {
                    if name == "px" && args.first().is_some_and(numeric) {
                        self.count("raw-px");
                    }
                    if matches!(name.as_str(), "rgb" | "rgba")
                        && args
                            .first()
                            .is_some_and(|arg| arg.to_token_stream().to_string().starts_with("0x"))
                    {
                        self.count("raw-color");
                    }
                    if matches!(name.as_str(), "hsl" | "hsla") && args.iter().any(numeric) {
                        self.count("raw-color");
                    }
                }
            }
        }
        for token in tokens {
            if let TokenTree::Group(group) = token {
                self.styles(group.stream());
            }
        }
    }
    fn macro_dispatch(&mut self, tokens: TokenStream, all_strings: bool) {
        let tokens: Vec<_> = tokens.into_iter().collect();
        for (i, token) in tokens.iter().enumerate() {
            if let TokenTree::Literal(lit) = token
                && syn::parse_str::<syn::LitStr>(&lit.to_string()).is_ok()
            {
                let next = tokens
                    .get(i + 1)
                    .map(ToString::to_string)
                    .unwrap_or_default();
                let second = tokens
                    .get(i + 2)
                    .map(ToString::to_string)
                    .unwrap_or_default();
                let prev = i
                    .checked_sub(1)
                    .and_then(|i| tokens.get(i))
                    .map(ToString::to_string)
                    .unwrap_or_default();
                let before = i
                    .checked_sub(2)
                    .and_then(|i| tokens.get(i))
                    .map(ToString::to_string)
                    .unwrap_or_default();
                if all_strings
                    || next == "|"
                    || (next == "=" && second == ">")
                    || (next == "=" && second == "=")
                    || (next == "!" && second == "=")
                    || (prev == "=" && matches!(before.as_str(), "=" | "!"))
                {
                    self.count("string-dispatch");
                }
            }
            if let TokenTree::Group(group) = token {
                self.macro_dispatch(group.stream(), all_strings);
            }
        }
    }
    fn function(&mut self, signature: &syn::Signature, block: &syn::Block) {
        let lines = block.span().end().line - signature.span().start().line + 1;
        if lines > FUNCTION_LINES {
            let key = format!(
                "{}:function-lines:{}::{}",
                self.path,
                self.scope.join("::"),
                signature.ident
            );
            // Duplicate cfg implementations share a budget using their maximum.
            self.counts
                .entry(key)
                .and_modify(|n| *n = (*n).max(lines))
                .or_insert(lines);
        }
    }
}
fn numeric(expr: &syn::Expr) -> bool {
    match expr {
        syn::Expr::Lit(syn::ExprLit {
            lit: syn::Lit::Int(_) | syn::Lit::Float(_),
            ..
        }) => true,
        syn::Expr::Unary(expr) => numeric(&expr.expr),
        syn::Expr::Paren(expr) => numeric(&expr.expr),
        _ => false,
    }
}
fn string(expr: &syn::Expr) -> bool {
    matches!(
        expr,
        syn::Expr::Lit(syn::ExprLit {
            lit: syn::Lit::Str(_),
            ..
        })
    )
}
impl<'ast> Visit<'ast> for Inspector<'_> {
    fn visit_item_mod(&mut self, node: &'ast syn::ItemMod) {
        self.scope.push(node.ident.to_string());
        visit::visit_item_mod(self, node);
        self.scope.pop();
    }
    fn visit_item_impl(&mut self, node: &'ast syn::ItemImpl) {
        self.scope.push(format!(
            "impl {} for {}",
            node.trait_
                .as_ref()
                .map(|(_, path, _)| path.to_token_stream().to_string())
                .unwrap_or_default(),
            node.self_ty.to_token_stream()
        ));
        visit::visit_item_impl(self, node);
        self.scope.pop();
    }
    fn visit_item_trait(&mut self, node: &'ast syn::ItemTrait) {
        self.scope.push(node.ident.to_string());
        visit::visit_item_trait(self, node);
        self.scope.pop();
    }
    fn visit_item_fn(&mut self, node: &'ast syn::ItemFn) {
        self.function(&node.sig, &node.block);
        self.scope.push(node.sig.ident.to_string());
        visit::visit_item_fn(self, node);
        self.scope.pop();
    }
    fn visit_impl_item_fn(&mut self, node: &'ast syn::ImplItemFn) {
        self.function(&node.sig, &node.block);
        self.scope.push(node.sig.ident.to_string());
        visit::visit_impl_item_fn(self, node);
        self.scope.pop();
    }
    fn visit_trait_item_fn(&mut self, node: &'ast syn::TraitItemFn) {
        if let Some(block) = &node.default {
            self.function(&node.sig, block);
        }
        visit::visit_trait_item_fn(self, node);
    }
    fn visit_macro(&mut self, node: &'ast syn::Macro) {
        if self.app {
            self.macro_dispatch(node.tokens.clone(), node.path.is_ident("matches"));
        }
        visit::visit_macro(self, node);
    }
    fn visit_pat(&mut self, node: &'ast syn::Pat) {
        if self.app
            && matches!(
                node,
                syn::Pat::Lit(syn::ExprLit {
                    lit: syn::Lit::Str(_),
                    ..
                })
            )
        {
            self.count("string-dispatch");
        }
        visit::visit_pat(self, node);
    }
    fn visit_expr_binary(&mut self, node: &'ast syn::ExprBinary) {
        if self.app
            && matches!(node.op, syn::BinOp::Eq(_) | syn::BinOp::Ne(_))
            && (string(&node.left) || string(&node.right))
        {
            self.count("string-dispatch");
        }
        visit::visit_expr_binary(self, node);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn scan(source: &str) -> Counts {
        let mut counts = Counts::new();
        inspect("crates/quantum-app/src/ui.rs", source, &mut counts).unwrap();
        counts
    }
    #[test]
    fn real_renames_preserve_budgets_without_allowing_new_or_larger_debt() {
        let previous = Counts::from([("old.rs:raw-px".into(), 3)]);
        let renamed = remap_counts(previous.clone(), "R095\0old.rs\0new.rs\0").unwrap();
        assert!(no_growth(&Counts::from([("new.rs:raw-px".into(), 3)]), &renamed).is_empty());
        assert!(!no_growth(&Counts::from([("new.rs:raw-px".into(), 4)]), &renamed).is_empty());
        assert!(!no_growth(&Counts::from([("new.rs:raw-color".into(), 1)]), &renamed).is_empty());
        assert!(!no_growth(&Counts::from([("new.rs:raw-px".into(), 3)]), &previous).is_empty());
        assert!(remap_counts(previous.clone(), "R100\0old.rs\0").is_err());
        assert!(remap_counts(previous, "A\0old.rs\0new.rs\0").is_err());
        let collision = Counts::from([("old.rs:raw-px".into(), 3), ("new.rs:raw-px".into(), 2)]);
        assert!(remap_counts(collision, "R100\0old.rs\0new.rs\0").is_err());
    }

    #[test]
    fn base_revision_fixture_bootstraps_only_an_absent_file() {
        use crate::{cmd::Output, testing::RecordingRunner, workspace::Workspace};
        let workspace = Workspace::discover().unwrap();
        for (tree, contents, expected) in [
            ("", None, None),
            (
                ALLOWLIST,
                Some(r#"{"old:raw-px":2}"#),
                Some(Counts::from([("old:raw-px".into(), 2)])),
            ),
        ] {
            let runner = RecordingRunner::stdout(b"base-sha\n");
            runner.results.borrow_mut().push_back(Ok(Output {
                stdout: tree.as_bytes().to_vec(),
                ..Output::default()
            }));
            if let Some(contents) = contents {
                runner.results.borrow_mut().push_back(Ok(Output {
                    stdout: contents.as_bytes().to_vec(),
                    ..Output::default()
                }));
            }
            let context = Context {
                workspace: &workspace,
                runner: &runner,
                strict: true,
            };
            assert_eq!(previous_baseline(&context, "pr-base").unwrap(), expected);
            runner.assert_args(0, &["rev-parse", "--verify", "pr-base^{commit}"]);
            runner.assert_args(1, &["ls-tree", "--name-only", "base-sha", "--", ALLOWLIST]);
        }
        let runner = RecordingRunner::default();
        runner
            .results
            .borrow_mut()
            .push_back(Err(std::io::Error::other("missing base revision")));
        let context = Context {
            workspace: &workspace,
            runner: &runner,
            strict: true,
        };
        assert!(previous_baseline(&context, "missing").is_err());
    }
    #[test]
    fn oversized_function_fixes_require_reducing_the_baseline() {
        let source = |n| format!("fn big() {{\n{}}}", "// line\n".repeat(n));
        let old = scan(&source(110));
        let smaller = scan(&source(105));
        assert!(ratchet(&old, &old).is_empty());
        assert!(!ratchet(&smaller, &old).is_empty());
        assert!(ratchet(&smaller, &smaller).is_empty());
        assert!(!ratchet(&scan(&source(115)), &old).is_empty());
        assert!(scan(&source(98)).is_empty());
        assert!(!ratchet(&scan(&source(98)), &old).is_empty());
    }
    #[test]
    fn allowlist_cannot_be_expanded_with_source_changes() {
        let previous = Counts::from([("file:raw-px".into(), 3)]);
        assert!(no_growth(&previous, &previous).is_empty());
        assert!(no_growth(&Counts::new(), &previous).is_empty());
        assert!(no_growth(&Counts::from([("file:raw-px".into(), 2)]), &previous).is_empty());
        assert!(!no_growth(&Counts::from([("file:raw-px".into(), 4)]), &previous).is_empty());
        assert!(!no_growth(&Counts::from([("new:raw-px".into(), 1)]), &previous).is_empty());
    }
    #[test]
    fn macro_tokens_are_checked_without_counting_comments_or_strings() {
        let counts = scan(
            r#"fn ui() { custom! { px(4.0); match action { "save" => (), _ => () } } matches!(action, "open" | "quit"); }"#,
        );
        assert_eq!(counts.values().sum::<usize>(), 4);
    }
    #[test]
    fn styling_and_dispatch_fixture() {
        let counts = scan(
            r#"fn ui(id: &str) { /* px(3) */ let _ = "rgb(0xff)"; px(2.0); gpui::px(-3.0); px(metrics::WIDTH); rgb(0xff00ff); rgba(color); hsla(0., 1., 1., 1.); match id { "save" | "open" => (), _ => () }; if id == "quit" {} }"#,
        );
        assert_eq!(counts.values().sum::<usize>(), 7);
        assert_eq!(counts["crates/quantum-app/src/ui.rs:raw-px"], 2);
        assert_eq!(counts["crates/quantum-app/src/ui.rs:raw-color"], 2);
        assert_eq!(counts["crates/quantum-app/src/ui.rs:string-dispatch"], 3);
        let mut shared = Counts::new();
        inspect(
            "crates/quantum-ui/src/tokens.rs",
            "fn tokens() { px(2.0); }",
            &mut shared,
        )
        .unwrap();
        assert!(shared.is_empty());
    }
    #[test]
    fn size_limits_count_syntax_not_braces_in_strings() {
        let source = format!(
            "impl A {{ fn big() {{\n{}\nlet _ = \"}}\";\n}} }}",
            "// line\n".repeat(110)
        );
        let counts = scan(&source);
        assert_eq!(counts.len(), 1);
        assert!(counts.keys().next().unwrap().contains("big"));
        let counts = scan(&format!("{}fn small() {{}}", "// line\n".repeat(1001)));
        assert_eq!(counts.values().copied().collect::<Vec<_>>(), [1002]);
        assert!(inspect("broken.rs", "fn {", &mut Counts::new()).is_err());
    }
    #[test]
    fn ratchet_requires_exact_remaining_debt() {
        let old = Counts::from([("fixture:raw-px".into(), 2)]);
        assert!(ratchet(&old, &old).is_empty());
        for n in [0, 1, 3] {
            let actual = Counts::from([("fixture:raw-px".into(), n)]);
            assert!(!ratchet(&actual, &old).is_empty());
        }
        assert!(!ratchet(&Counts::new(), &old).is_empty());
        assert!(!ratchet(&old, &Counts::new()).is_empty());
        assert!(!ratchet(&Counts::new(), &Counts::from([("stale".into(), 0)])).is_empty());
    }
}

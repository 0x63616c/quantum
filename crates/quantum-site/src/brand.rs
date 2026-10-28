//! Resolve product tokens at the output boundary; encoded installation data uses
//! the typed inventory before serialization.
use quantum_brand::Brand;
use std::{error::Error, path::Path, sync::LazyLock};

const INVENTORY: &str = include_str!("../../../brand.toml");
const MANIFEST: &str = include_str!("../../../Cargo.toml");
static BRAND: LazyLock<Brand> = LazyLock::new(|| {
    // main validates these immutable embedded inputs before generating output.
    #[expect(
        clippy::expect_used,
        reason = "embedded inputs validated before generation and in tests"
    )]
    Brand::parse(INVENTORY, MANIFEST).expect("valid embedded brand metadata")
});

pub fn validate() -> Result<(), Box<dyn Error>> {
    Brand::parse(INVENTORY, MANIFEST)?;
    Ok(())
}

pub fn get() -> &'static Brand {
    &BRAND
}

pub fn expand(source: &str) -> String {
    expand_values(source, false)
}

fn expand_values(source: &str, html: bool) -> String {
    let brand = get();
    let mut text = source.to_owned();
    for (key, value) in [
        ("download_url", brand.download_url()),
        (
            "repo_url",
            brand.identifiers.github_repo_url.current.clone(),
        ),
        ("site_url", brand.site_url()),
        ("product", brand.product.display_name.current.clone()),
        ("tagline", brand.product.tagline.current.clone()),
        ("app_bundle", brand.identifiers.app_bundle.current.clone()),
        (
            "release_asset",
            brand.identifiers.release_asset.current.clone(),
        ),
        ("version", brand.version.clone()),
    ] {
        let value = if html { crate::escape(&value) } else { value };
        text = text.replace(&format!("@@{key}@@"), &value);
    }
    text
}

pub fn write(path: impl AsRef<Path>, text: impl AsRef<[u8]>) -> Result<(), Box<dyn Error>> {
    let text = std::str::from_utf8(text.as_ref())?;
    let expanded = expand_values(
        text,
        path.as_ref().extension().is_some_and(|ext| ext == "html"),
    );
    if expanded.contains("@@") {
        return Err("unresolved brand token".into());
    }
    quantum_format::storage::safe_write(path, expanded)?;
    Ok(())
}

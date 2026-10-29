//! Contracts for the automatic tag-to-publication release workflow.

use serde_json::Value;

const SECRETS: [&str; 6] = [
    "MACOS_CERTIFICATE_P12_BASE64",
    "MACOS_CERTIFICATE_PASSWORD",
    "APPLE_API_KEY_P8_BASE64",
    "APPLE_API_KEY_ID",
    "APPLE_API_ISSUER_ID",
    "SPARKLE_ED_PRIVATE_KEY",
];

fn workflow() -> Value {
    serde_saphyr::from_str(include_str!(
        "../../../../.github/workflows/release-macos.yml"
    ))
    .unwrap()
}

#[test]
fn credentials_fail_closed_before_build_without_disclosing_values() {
    let workflow = workflow();
    let steps = workflow["jobs"]["dmg"]["steps"].as_array().unwrap();
    let index = steps
        .iter()
        .position(|step| step["name"] == "Validate release credentials")
        .unwrap();
    assert!(
        index
            < steps
                .iter()
                .position(|step| step.get("uses").is_some())
                .unwrap()
    );
    let step = &steps[index];
    for secret in SECRETS {
        assert_eq!(step["env"][secret], format!("${{{{ secrets.{secret} }}}}"));
    }
    // Execute the actual workflow script with synthetic values only. Test each
    // unset credential as well as the fully configured case.
    for missing in SECRETS.into_iter().map(Some).chain([None]) {
        let mut command = std::process::Command::new("bash");
        command.args([
            "--noprofile",
            "--norc",
            "-e",
            "-c",
            step["run"].as_str().unwrap(),
        ]);
        command.env_clear();
        for secret in SECRETS {
            if missing != Some(secret) {
                command.env(secret, "fixture-value");
            }
        }
        let output = command.output().unwrap();
        assert_eq!(output.status.success(), missing.is_none());
        let stderr = String::from_utf8(output.stderr).unwrap();
        assert!(output.stdout.is_empty());
        assert!(!stderr.contains("fixture-value"));
        if let Some(secret) = missing {
            assert_eq!(
                stderr,
                format!("Missing required release secret: {secret}\n")
            );
        }
    }
    for step in steps {
        if step["run"]
            .as_str()
            .is_some_and(|script| script.contains("cargo "))
        {
            assert!(
                step.get("env").is_none(),
                "builds must not receive signing secrets"
            );
        }
    }
}

#[test]
fn stable_tags_automatically_package_and_publish_all_update_assets() {
    let workflow = workflow();
    assert_eq!(workflow["on"]["push"]["tags"][0], "v*");
    let job = &workflow["jobs"]["dmg"];
    assert_eq!(job["env"]["CARGO_BUILD_JOBS"], 4);
    assert!(
        job["if"]
            .as_str()
            .unwrap()
            .contains("github.actor == '0x63616c'")
    );
    let steps = job["steps"].as_array().unwrap();
    let scripts = steps
        .iter()
        .filter_map(|step| step["run"].as_str())
        .collect::<Vec<_>>()
        .join("\n");
    let mut end = 0;
    for operation in [
        "version --check-tag",
        "macos package --skip-build",
        "macos release-notes",
        "macos fetch-appcast",
        "macos appcast",
        "test \"$APPCAST_GENERATED\" = true",
        "gh release create",
        "gh release upload",
        "gh release edit",
    ] {
        end += scripts[end..].find(operation).unwrap() + operation.len();
    }
    for asset in [
        "dist/Quantum.dmg",
        "dist/Quantum.dmg.sha256",
        "dist/Quantum.html",
        "site/appcast.xml",
    ] {
        assert!(scripts.contains(asset));
    }
    assert!(!scripts.contains("--clobber"));
}

#[test]
fn optional_report_endpoint_reaches_only_the_release_app_build() {
    let workflow = workflow();
    let job = &workflow["jobs"]["dmg"];
    assert_eq!(
        job["env"]["APP_REPORT_ENDPOINT"],
        "${{ vars.QUANTUM_REPORT_ENDPOINT }}"
    );
    assert!(job["env"].get("QUANTUM_REPORT_ENDPOINT").is_none());
    let steps = job["steps"].as_array().unwrap();
    let build = steps
        .iter()
        .find(|step| step["name"] == "Build app without signing credentials")
        .unwrap();
    assert!(build.get("env").is_none());
    assert!(build["run"].as_str().unwrap().contains(
        "QUANTUM_REPORT_ENDPOINT=\"$APP_REPORT_ENDPOINT\" cargo build --release --locked -p quantum-app --bin quantum"
    ));
    for step in steps.iter().filter(|step| step["name"] != build["name"]) {
        assert!(
            !step["run"]
                .as_str()
                .unwrap_or_default()
                .contains("QUANTUM_REPORT_ENDPOINT")
        );
    }
}

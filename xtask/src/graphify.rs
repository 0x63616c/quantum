//! Pinned, local code-graph refresh through uv's isolated tool environment.

use xshell::{Shell, cmd};

use crate::{
    Error, Result,
    cmd::{Command, Runner},
    workspace::Workspace,
};

const VERSION: &str = "0.9.97";

/// Refresh the repository graph, refusing an unexpected distribution version.
///
/// Never uses a graphify executable from the caller's PATH. Output and cache stay
/// in the ignored `graphify-out/` directory; uv manages its own tool cache.
pub fn refresh(workspace: &Workspace, runner: &dyn Runner) -> Result<()> {
    let sh = Shell::new().map_err(|error| Error::Invalid(error.to_string()))?;
    sh.change_dir(workspace.root());
    let package = format!("graphifyy=={VERSION}");
    let probe = "import importlib.metadata; print(importlib.metadata.version('graphifyy'))";
    let command = Command::from(cmd!(sh, "uv tool run --from {package} python -c {probe}"));
    let actual = command.read(runner).map_err(|error| {
        Error::Invalid(format!(
            "graphify requires uv and {package}; install uv and retry: {error}"
        ))
    })?;
    if actual != VERSION {
        return Err(Error::Invalid(format!(
            "refusing graphify refresh: expected {package}, found {actual:?}"
        )));
    }
    Command::from(cmd!(sh, "uv tool run --from {package} graphify update .")).run(runner)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{cmd::Output, testing::RecordingRunner};

    #[test]
    fn refresh_uses_pinned_environment_and_repository_root() {
        let runner = RecordingRunner::stdout(format!("{VERSION}\n").as_bytes());
        runner.results.borrow_mut().push_back(Ok(Output::default()));
        let workspace = Workspace::discover().unwrap();
        refresh(&workspace, &runner).unwrap();
        runner.assert_args(
            1,
            &[
                "tool",
                "run",
                "--from",
                "graphifyy==0.9.97",
                "graphify",
                "update",
                ".",
            ],
        );
        for (command, _) in runner.commands.borrow().iter() {
            assert_eq!(command.program(), "uv");
            assert_eq!(command.directory(), Some(workspace.root()));
        }
    }

    #[test]
    fn wrong_or_missing_version_never_runs_update() {
        for version in ["0.9.96", "", "0.9.970"] {
            let runner = RecordingRunner::stdout(version.as_bytes());
            let error = refresh(&Workspace::discover().unwrap(), &runner).unwrap_err();
            assert!(error.to_string().contains("expected graphifyy==0.9.97"));
            assert_eq!(runner.commands.borrow().len(), 1);
        }
    }

    #[test]
    fn missing_uv_has_actionable_error() {
        let runner = RecordingRunner::default();
        runner
            .results
            .borrow_mut()
            .push_back(Err(std::io::Error::from(std::io::ErrorKind::NotFound)));
        let error = refresh(&Workspace::discover().unwrap(), &runner).unwrap_err();
        assert!(error.to_string().contains("install uv and retry"));
        assert_eq!(runner.commands.borrow().len(), 1);
    }

    #[test]
    fn update_failure_is_propagated() {
        let runner = RecordingRunner::stdout(VERSION.as_bytes());
        runner
            .results
            .borrow_mut()
            .push_back(Err(std::io::Error::other("update failed")));
        assert!(refresh(&Workspace::discover().unwrap(), &runner).is_err());
    }
}

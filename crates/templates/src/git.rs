use std::{io::ErrorKind, path::Path, process::Stdio};

use anyhow::Context;

// TODO: the following and the second half of plugins/git.rs are duplicates

pub(crate) enum GitError {
    ProgramFailed(Vec<u8>),
    ProgramNotFound,
    Other(anyhow::Error),
}

impl std::fmt::Display for GitError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ProgramNotFound => f.write_str("`git` command not found - is git installed?"),
            Self::Other(e) => e.fmt(f),
            Self::ProgramFailed(stderr) => match std::str::from_utf8(stderr) {
                Ok(s) => f.write_str(s),
                Err(_) => f.write_str("(cannot get error)"),
            },
        }
    }
}

pub(crate) trait UnderstandGitResult {
    fn understand_git_result(self) -> Result<Vec<u8>, GitError>;
}

impl UnderstandGitResult for Result<std::process::Output, std::io::Error> {
    fn understand_git_result(self) -> Result<Vec<u8>, GitError> {
        match self {
            Ok(output) => {
                if output.status.success() {
                    Ok(output.stdout)
                } else {
                    Err(GitError::ProgramFailed(output.stderr))
                }
            }
            Err(e) => match e.kind() {
                // TODO: consider cases like insufficient permission?
                ErrorKind::NotFound => Err(GitError::ProgramNotFound),
                _ => {
                    let err = anyhow::Error::from(e).context("Failed to run `git` command");
                    Err(GitError::Other(err))
                }
            },
        }
    }
}

pub(crate) async fn is_in_git_repo(dir: &Path) -> anyhow::Result<bool> {
    let mut cmd = tokio::process::Command::new("git");
    cmd.arg("-C")
        .arg(dir)
        .arg("rev-parse")
        .arg("--git-dir")
        .stdout(Stdio::null())
        .stderr(Stdio::null());

    let status = cmd
        .status()
        .await
        .context("checking if new app is in a git repo")?;
    Ok(status.success())
}

pub(crate) async fn init_git_repo(dir: &Path) -> Result<(), GitError> {
    let mut cmd = tokio::process::Command::new("git");
    cmd.arg("-C").arg(dir).arg("init");

    let result = cmd.output().await;
    result.understand_git_result().map(|_| ())
}

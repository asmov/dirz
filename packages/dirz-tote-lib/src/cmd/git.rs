use std::path::{PathBuf, Path};
use std::process;
use std::process::Stdio;
use crate::error::*;

pub trait GitCLI {
    fn init(&self) -> Result<bool>;
    fn add_path(&mut self, pathspec: &Path) -> Result<bool>;
    //fn remove();
    //fn commit();
    //fn push();
    //fn pull();
    //fn branch();
    //fn tag();
}

pub struct Git {
    repository_path: PathBuf,
    git_config_path: PathBuf,
}

impl Git {
    const GIT_CMD: &'static str = "git";
    const GIT_CMD_ERROR: &'static str = "Failed to run command: git";
    const GIT_ARG_PORCELAIN: &'static str = "--porcelain";
    const GIT_OP_INIT: &'static str = "init";
    const GIT_OP_ADD: &'static str = "add";

    pub fn new(repository_path: &Path) -> Self {
        Git {
            repository_path: repository_path.to_path_buf(),
            git_config_path: path!(repository_path, ".git", "config")
        }
    }

    fn create_cmd(&self) -> process::Command {
        let mut cmd= process::Command::new(Self::GIT_CMD);
        cmd
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        cmd
    }

}

impl GitCLI for Git {
    fn init(&self) -> Result<bool> {
        if self.git_config_path.exists() {
            return Ok(false)
        } else if self.repository_path.is_file() {
            bail!("Repository path is not a directory: {}", self.repository_path.to_str().unwrap());
        }

        let output = self.create_cmd()
            .arg(Self::GIT_OP_INIT).arg(&self.repository_path)
            //.stdout(Stdio::null())
            .output()
            .expect(Self::GIT_CMD_ERROR);

        if !output.status.success() {
            bail!(String::from_utf8_lossy(&output.stderr).trim())
        }

        Ok(true)
    }

    fn add_path(&mut self, _pathspec: &Path) -> Result<bool> {
        todo!()
    }
}

pub struct Pathspec {
    directory_prefix: PathBuf,
    magic_signature:
}

pub enum MagicSignature {
    Top,
    Literal,
    ICase,
    Glob,
    Attr,
    Exclude,
    Parent,
    Pickaxe,
    Plumbing,
    Porcelain,
    PerWorktreeRef,
    Pseudoref,
    Pull,
    Push,
    Reachable,
    Rebase,
    Ref,
    RefLog,
    RefSpec,
    RemoteRepository,
    RemoteTrackingBranch,
    Repository,
    Resolve,
    Revision,
    Re


}


#[cfg(test)]
mod tests {
    use std::path::{PathBuf, Path};
    use crate::cmd::git::{Git, GitCLI};
    use crate::cmd::test;

    const REPOSITORY_RELPATH: &'static str = "cmd/git";
    const GIT_CONFIG_RELPATH: &'static str = "cmd/git/.git/config";

    fn setup() -> (PathBuf, Git){
        let tmp_dir = test::setup_tmpdir();
        let git_cmd = Git::new(&PathBuf::from(path!(&tmp_dir, REPOSITORY_RELPATH)));
        (tmp_dir, git_cmd)
    }

    fn teardown(tmp_dir: &Path) {
        test::teardown_tmpdir(tmp_dir)
    }

    #[test]
    fn new() {
        let (tmp_dir, git_cmd) = setup();
        assert_eq!(path!(&tmp_dir, REPOSITORY_RELPATH).to_str().unwrap(),
                   git_cmd.repository_path.to_str().unwrap(), "Repository path should match");
        assert_eq!(path!(&tmp_dir, GIT_CONFIG_RELPATH).to_str().unwrap(),
                   git_cmd.git_config_path.to_str().unwrap(), "Git config file path should match");
        teardown(&tmp_dir);
    }

    #[test]
    fn init() {
        let (tmp_dir, git_cmd) = setup();
        git_cmd.init().expect("Git should init successfully");
        assert!(git_cmd.git_config_path.exists(), "Git config file should exist after init");
        teardown(&tmp_dir);
    }
}
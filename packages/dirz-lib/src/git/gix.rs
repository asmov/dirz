use std::path::{PathBuf, Path};
use crate::path;
use super::{RepoAdapterFactoryFn, RepoAdapter, NewOperation};

pub(super) struct Repo {
    repo: gix::Repository,
}

impl RepoAdapter for Repo {}

impl Repo {
    pub(super) fn new(dir: &Path, op: NewOperation) -> Result<Box<dyn RepoAdapter>, anyhow::Error> {
        Ok(Box::new(Self {
            repo: match op {
                NewOperation::Init => gix::init(dir)?,
                NewOperation::Open => gix::open(dir)?,
                NewOperation::Clone { from } => {
                    gix::interrupt::init_handler(|| {})?;  // do we need this?
                    gix::prepare_clone(gix::url::parse(from.as_bytes().into()).unwrap(), dir)?
                        .fetch_then_checkout(gix::progress::Discard, &gix::interrupt::IS_INTERRUPTED)?.0
                        .main_worktree(gix::progress::Discard, &gix::interrupt::IS_INTERRUPTED)?.0
                }
            }
        }))
    }
}

#[cfg(test)]
mod tests {
    use std::path::{PathBuf, Path};
    use crate::path;
    use super::*;
    use crate::testing::Testing;

    const DOT_GIT: &'static str = ".git";
    const CONFIG: &'static str = "config";

    const FIXTURE_DIRPATH: &'static str = "tests/fixtures/git";
    const FIXTURE_TEST_NAME: &'static str = "git-test";

    lazy_static::lazy_static! {
        static ref TESTING: Testing = {
            //shutdown_hooks::add_shutdown_hook(testing_teardown);
            Testing::setup("/tmp/dirz-lib-test/git/gix")
        };

        static ref FIXTURE_TMPDIR: PathBuf = {
            TESTING.create_subtest_dir_copy(&path!(FIXTURE_DIRPATH), FIXTURE_TEST_NAME)
        };
    }

    extern fn testing_teardown() {
        TESTING.teardown();
    }

    #[test]
    fn test_new_init() {
        let init_tmpdir = TESTING.create_subtest_dir("test_new_init");
        let repo = Repo::new(&init_tmpdir, NewOperation::Init).unwrap();
        assert!(init_tmpdir.join(DOT_GIT).join(CONFIG).exists(), ".git/config doesn't exist");
        let repo = Repo::new(&init_tmpdir, NewOperation::Open).unwrap();
    }

    #[test]
    fn test_new_open() {
        let init_tmpdir = TESTING.create_subtest_dir("test_new_open");
        let repo = Repo::new(&init_tmpdir, NewOperation::Init).unwrap();
        assert!(init_tmpdir.join(DOT_GIT).join(CONFIG).exists(), ".git/config doesn't exist");

        let repo = Repo::new(&init_tmpdir, NewOperation::Open).unwrap();
    }

    #[test]
    fn test_new_clone() {
        const GITHUB_URL: &'static str = "https://github.com/asmov/dirz.git";

        let init_tmpdir = TESTING.create_subtest_dir("test_new_clone");
        let repo = Repo::new(&init_tmpdir, NewOperation::Clone{from: String::from(GITHUB_URL)}).unwrap();
        assert!(init_tmpdir.join("Cargo.toml").exists());

        let repo = Repo::new(&init_tmpdir, NewOperation::Open).unwrap();
    }

}

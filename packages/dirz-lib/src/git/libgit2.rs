use std::path::{PathBuf, Path};
use crate::path;
use super::{RepoAdapterFactoryFn, RepoAdapter, NewOperation};

pub(super) struct Repo {
    repo: git2::Repository,
}

impl RepoAdapter for Repo {}

impl Repo {
    pub(super) fn new(dir: &Path, op: NewOperation) -> Result<Box<dyn RepoAdapter>, anyhow::Error> {
        Ok(Box::new(Self {
            repo: match op {
                NewOperation::Init => git2::Repository::init(dir)?,
                NewOperation::Open => git2::Repository::open(dir)?,
                NewOperation::Clone { from } => unimplemented!("Cloning via libgit2 is not supported!")
            }
        }))
    }
}

/*impl Repo {
    fn find_last_commit(repo: &git2::Repository) -> Result<git2::Commit, git2::Error> {
        let obj = repo.head()?.resolve()?.peel(git2::ObjectType::Commit)?;
        obj.into_commit().map_err(|_| git2::Error::from_str("Couldn't find commit"))
    }

    fn init_commit(gitrepo: &super::Repo, repo: &git2::Repository, path: &Path) -> Result<git2::Oid, git2::Error> {
        let paths: Vec<PathBuf> = vec![  // todo: list dir
            PathBuf::from(".cabinet/schema.toml"),
        ];

        //gitrepo.init_commit(&path!(".gitignore"))?;
        //gitrepo.add_and_commit("added schema", paths)?;


        let mut index = repo.index()?;  // retrieve the index file for the repository
        index.add_path(path).unwrap();
        let oid = index.write_tree()?;  // recursively writes an index
        let signature = git2::Signature::now("Testing", "testing@localhost")?;
        let tree = repo.find_tree(oid)?;
        repo.commit(Some("HEAD"), //  point HEAD to our new commit
                    &signature, // author
                    &signature, // committer
                    "Initial commit", // commit message
                    &tree, // tree
                    &[]) // parents
    }

    fn add_and_commit(repo: &git2::Repository, message: &str, paths: Vec<PathBuf>) -> Result<git2::Oid, git2::Error> {
        let mut index = repo.index()?;  // retrieve the index file for the repository

        for path in paths {
            if !repo.is_path_ignored(&path).unwrap() {
                index.add_path(&path)?;  // doesn't follow .gitignore rules. path is relative to repo working dir.
            }
        }

        index.write().unwrap();
        let oid = index.write_tree()?;  // recursively writes an index
        let signature = git2::Signature::now("Testing", "testing@localhost")?;
        let parent_commit = gitrepo.find_last_commit(&repo)?;
        let tree = repo.find_tree(oid)?;
        repo.commit(Some("HEAD"), //  point HEAD to our new commit
                    &signature, // author
                    &signature, // committer
                    message, // commit message
                    &tree, // tree
                    &[&parent_commit])
    }

}*/

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
            Testing::setup("/tmp/dirz-lib-test/git/libgit2")
        };

        static ref FIXTURE_TMPDIR: PathBuf = {
            TESTING.create_subtest_dir_copy(&path!(FIXTURE_DIRPATH), FIXTURE_TEST_NAME)
        };
    }

    extern fn testing_teardown() {
        TESTING.teardown();
    }

    #[test]
    fn test_new_init_open() {
        let init_tmpdir = TESTING.create_subtest_dir("test_new_init_open");
        let repo = Repo::new(&init_tmpdir, NewOperation::Init).unwrap();
        assert!(init_tmpdir.join(DOT_GIT).join(CONFIG).exists(), ".git/config doesn't exist");

        let repo = Repo::new(&init_tmpdir, NewOperation::Open).unwrap();
    }

    #[test]
    #[should_panic]
    fn test_new_clone() {
        const GITHUB_URL: &'static str = "https://github.com/asmov/dirz.git";
        let init_tmpdir = TESTING.create_subtest_dir("test_new_clone");
        Repo::new(&init_tmpdir, NewOperation::Clone{from: String::from(GITHUB_URL)}).unwrap();
    }

}

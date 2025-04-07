use std::hash::Hash;
use std::path::{PathBuf,Path};
use std::collections::HashMap;

#[cfg(feature = "gix")]
pub(super) mod gix;

#[cfg(feature = "libgit2")]
pub(super) mod libgit2;


trait RepoAdapter {
    //fn new(dir: &Path, op: NewOperation) -> Result<Box<Self>, anyhow::Error>;
}

#[derive(Clone, Eq, PartialEq, Hash)]
enum Library {
    Git,
    LibGit2,
    Gix
}

type RepoAdapterFactoryFn = Box<dyn Fn(&Path, NewOperation) -> Result<Box<dyn RepoAdapter>, anyhow::Error> + 'static + Sync> ;

struct Functionality {
    init: Library,
    open: Library,
    clone: Library,
    root_commit: Library,
    find_last_commit: Library,
    add_commit: Library,
}

lazy_static::lazy_static! {
    static ref ADAPTER_FACTORIES: HashMap<Library, RepoAdapterFactoryFn> = {
        let mut map: HashMap<Library, RepoAdapterFactoryFn> = HashMap::new();
        #[cfg(feature = "libgit2")]
        map.insert(Library::LibGit2, Box::new(libgit2::Repo::new));
        #[cfg(feature = "gix")]
        map.insert(Library::Gix, Box::new(gix::Repo::new));
        map
    };
}


const FUNCTIONALITY: Functionality = if cfg!(feature = "gix") {
    Functionality {
        init: Library::Gix,
        open: Library::Gix,
        clone: Library::Gix,
        root_commit: Library::LibGit2,
        find_last_commit: Library::LibGit2,
        add_commit: Library::LibGit2,
    }
} else {
    Functionality {
        init: Library::LibGit2,
        open: Library::LibGit2,
        clone: Library::LibGit2,
        root_commit: Library::LibGit2,
        find_last_commit: Library::LibGit2,
        add_commit: Library::LibGit2,
    }
};

struct Repo {
    dir: PathBuf,
    adapters: HashMap<Library, Box<dyn RepoAdapter>>
}

enum NewOperation {
    Open,
    Init,
    Clone { from: String }
}

impl Repo {
    fn init(dir: &Path) -> Result<Self, anyhow::Error> {
        Self::new(dir, NewOperation::Init)
    }

    fn new(dir: &Path, op: NewOperation) -> Result<Self, anyhow::Error> {
        let first_library = match op {
            NewOperation::Open => FUNCTIONALITY.open,
            NewOperation::Init => FUNCTIONALITY.init,
            NewOperation::Clone { from: _ } => FUNCTIONALITY.clone
        };

        let mut adapters: HashMap<Library, Box<dyn RepoAdapter>> = HashMap::new();
        adapters.insert(first_library.clone(), ADAPTER_FACTORIES.get(&first_library).unwrap()(dir, op)?);

        for (library, factory) in ADAPTER_FACTORIES.iter() {
            if *library != first_library {
                adapters.insert(library.clone(), factory(dir, NewOperation::Open)?);
            }
        }

        Ok(Self {
            dir: dir.to_owned(),
            adapters
        })
    }

}


#[cfg(test)]
mod tests {
    use std::path::{PathBuf, Path};
    use crate::path;
    use super::*;
    use crate::testing::Testing;


    const FIXTURE_DIRPATH: &'static str = "tests/fixtures/git";
    const FIXTURE_TEST_NAME: &'static str = "git-test";

    lazy_static::lazy_static! {
        static ref TESTING: Testing = {
            shutdown_hooks::add_shutdown_hook(testing_teardown);
            Testing::setup("/tmp/dirz-lib-test")
        };

        static ref FIXTURE_TMPDIR: PathBuf = {
            TESTING.create_subtest_dir_copy(&path!(FIXTURE_DIRPATH), FIXTURE_TEST_NAME)
        };
    }

    extern fn testing_teardown() {
        TESTING.teardown();
    }

    #[test]
    fn test_init() {
        let init_tmpdir = TESTING.create_subtest_dir("test-init");
        let repo = Repo::init(&init_tmpdir).unwrap();
        assert_eq!(repo.dir, init_tmpdir);
    }

    #[test]
    fn repo_trait() {
    }
}

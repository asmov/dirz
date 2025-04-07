use std::collections::BTreeMap;
use std::path::{PathBuf, Path};
use std::{fs, io};
use serde::{Deserialize, Serialize};

use crate::path;
use crate::schema;

const CABINET_DIRNAME: &'static str = ".cabinet";
const SCHEMA_FILENAME_SUFFIX: &'static str = ".schema.toml";
const SCHEMA_RELPATH: &'static str = ".cabinet/schema.toml";
const TREE_BIN_FILENAME: &'static str = "tree.bin";
const TREE_BIN_RELPATH: &'static str = ".cabinet/tree.bin";

pub trait NodeTrait: PartialEq {
    fn children(&self) -> &BTreeMap<String, Node>;
    fn add_node(&mut self, name: &str, node: Node);
    fn schema(&self) -> &schema::CabinetPath;
    fn antecedents(&self) -> &Vec<String>;
    fn parent<'a: 'c,'b: 'c,'c>(&'a self, tree: &'b Tree) -> Option<&'c Node>;
    //
    fn child(&self, relpath: &str) -> Option<&Node>;
    fn has_parent(&self) -> bool;
    fn has_child(&self, name: &str) -> bool;
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub enum Node {
    Path (FolderNode),
    ShardedPath (ShardedFolderNode)
}

#[derive(PartialEq,Debug)]
pub enum NodeType {
    Path,
    ShardedPath
}

impl Node {
    fn node_type(&self) -> NodeType {
        match self {
            Self::Path(_) => NodeType::Path,
            Self::ShardedPath(_) => NodeType::ShardedPath,
        }
    }
}

impl NodeTrait for Node {
    fn schema(&self) -> &schema::CabinetPath {
        match self {
            Self::Path(node) => node.schema(),
            Self::ShardedPath(node) => node.schema(),
        }
    }

    fn add_node(&mut self, name: &str, node: Node) {
        match self {
            Self::Path(parent) => parent.add_node(name, node),
            Self::ShardedPath(parent) => parent.add_node(name, node)
        }
    }

    fn children(&self) -> &BTreeMap<String, Node> {
       match self {
            Self::Path(node) => node.children(),
            Self::ShardedPath(node) => node.children()
        }
    }

    fn child(&self, name: &str) -> Option<&Node> {
       match self {
            Self::Path(node) => node.child(name),
            Self::ShardedPath(node) => node.child(name)
        }
    }

    fn has_parent(&self) -> bool {
        match self {
            Self::Path(node) => node.has_parent(),
            Self::ShardedPath(node) => node.has_parent(),
        }
    }

    fn has_child(&self, name: &str) -> bool {
        match self {
            Self::Path(node) => node.has_child(name),
            Self::ShardedPath(node) => node.has_child(name),
        }
    }

    fn antecedents(&self) -> &Vec<String> {
        match self {
            Self::Path(node) => node.antecedents(),
            Self::ShardedPath(node) => node.antecedents()
        }
    }

    fn parent<'a: 'c,'b: 'c,'c>(&'a self, tree: &'b Tree) -> Option<&'c Node> {
        tree.parent_for(self)
    }
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct FolderNode {
    name: String,
    dir_path: PathBuf,
    schema_filepath: PathBuf,
    schema: schema::CabinetPath,
    antecedents: Vec<String>,
    children: BTreeMap<String, Node>,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct ShardedFolderNode {
    name: String,
    schema_filepath: PathBuf,
    schema: schema::CabinetPath,
    antecedents: Vec<String>,
    children: BTreeMap<String, Node>,
}

impl NodeTrait for ShardedFolderNode {
    fn schema(&self) -> &schema::CabinetPath {
        &self.schema
    }

    fn add_node(&mut self, name: &str, node: Node) {
        self.children.insert(name.to_owned(), node);
    }

    fn children(&self) -> &BTreeMap<String, Node> {
        &self.children
    }

    fn child(&self, relpath: &str) -> Option<&Node> {
        self.children.get(relpath)
    }

    fn has_parent(&self) -> bool {
        !self.antecedents.is_empty()
    }

    fn has_child(&self, relpath: &str) -> bool {
        self.children.contains_key(relpath)
    }

    fn antecedents(&self) -> &Vec<String> {
        &self.antecedents
    }

    fn parent<'a: 'c,'b: 'c,'c>(&'a self, tree: &'b Tree) -> Option<&'c Node> {
        tree.parent_for(self)
    }
}

impl NodeTrait for FolderNode {
    fn schema(&self) -> &schema::CabinetPath {
        &self.schema
    }

    fn add_node(&mut self, name: &str, node: Node) {
        self.children.insert(name.to_owned(), node);
    }

    fn children(&self) -> &BTreeMap<String, Node> {
        &self.children
    }

    fn child(&self, relpath: &str) -> Option<&Node> {
        self.children.get(relpath)
    }

    fn has_parent(&self) -> bool {
        !self.antecedents.is_empty()
    }

    fn has_child(&self, name: &str) -> bool {
        self.children.contains_key(name)
    }

    fn antecedents(&self) -> &Vec<String> {
        &self.antecedents
    }

    fn parent<'a: 'c,'b: 'c,'c>(&'a self, tree: &'b Tree) -> Option<&'c Node> {
        tree.parent_for(self)
    }
}

#[derive(PartialEq, Debug)]
pub struct Tree {
    root: Node,
    basedir: PathBuf,
}

impl Tree {
    pub fn parent_for<'a: 'c,'b: 'c,'c, N: NodeTrait>(&'a self, node: &'b N) -> Option<&'c Node> {
        let relpaths = node.antecedents();
        if relpaths.is_empty() { // this should only be true for the root node
            //debug_assert!(self.root == node);
            return None;
        }

        let mut relpaths = relpaths.iter();

        // the first relpath is the name of the root node / cabinet / tree
        if cfg!(debug_assertions) {
            // assertion: self is the appropriate tree for the node provided
            debug_assert_eq!(&self.root.schema().name, relpaths.next().unwrap());
        } else {
            relpaths.next();
        }

        let mut antecedent_node = match relpaths.next() {
            Some(relpath) => self.root.child(relpath)
                .expect(&format!("Antecedent {} does not exist for: {}", relpath, node.schema().name)),
            None => return Some(&self.root)
        };

        for relpath in relpaths {
            antecedent_node = match antecedent_node.child(&relpath) {
                Some(ref node) => node,
                None => return None,
            };
        }

        Some(antecedent_node)
    }

    pub fn child_for(&self, path_str: &str) -> Option<&Node> {
        let mut node: &Node = &self.root;
        for path in path_str.split('/') {
            node = match node.child(path) {
                Some(ref n) => n,
                None => return None,
            }
        }

        Some(node)
    }

    pub fn read(dir: &Path) -> Result<Tree, io::Error> {
        let dir = fs::canonicalize(dir).unwrap();
        let root_node = Self::read_node(&dir, &PathBuf::new(), Vec::new(), None)?;

        Ok(Tree {
            root: root_node,
            basedir: dir.into()
        })
    }

    fn read_node(rootdir: &Path, reldir: &Path, antecedents: Vec<String>, sharded_schema_filepath: Option<PathBuf>)
            -> Result<Node, io::Error> {
        let sharded = sharded_schema_filepath.is_some();
        let schema_filepath = match sharded_schema_filepath {
            Some(path) => path,
            None => path!(rootdir, reldir, SCHEMA_RELPATH)
        };

        let schema: schema::CabinetPath = toml::from_str(&fs::read_to_string(&schema_filepath)?)?;

        let mut node = if !sharded {
            Node::Path(FolderNode {
                name: schema.name.to_string(),
                schema: schema,
                schema_filepath: path!(reldir, SCHEMA_RELPATH),
                dir_path: reldir.to_path_buf(),
                antecedents,
                children: BTreeMap::new()
            })
        } else {
            Node::ShardedPath(ShardedFolderNode {
                name: schema.name.to_string(),
                schema_filepath: path!(reldir, CABINET_DIRNAME, &schema.name, SCHEMA_FILENAME_SUFFIX),
                schema: schema,
                antecedents,
                children: BTreeMap::new()
            })
        };

        let mut child_antecedents = node.antecedents().clone();
        child_antecedents.push(String::from(&node.schema().name));

        for child_path in node.schema().folders.clone() {  // name => relative_path
            let child_reldir = path!(reldir, &child_path);
            let child = Self::read_node(rootdir, &child_reldir, child_antecedents.clone(), None)?;
            node.add_node(&child_path, child);
        }

        for shard_path in node.schema().sharded_folders.clone() {  // name => relative_path
            let child_schema_filepath= path!(rootdir, reldir, CABINET_DIRNAME, format!("{}{}", &shard_path, SCHEMA_FILENAME_SUFFIX));
            let child = Self::read_node(rootdir, &reldir, child_antecedents.clone(), Some(child_schema_filepath))?;
            node.add_node(&shard_path, child);
        }

        Ok(node)
    }

    pub fn save(&self) -> Result<(), anyhow::Error> {
        Self::write_root_encoded(&self.root, &path!(&self.basedir,TREE_BIN_RELPATH))
    }

    fn write_root_encoded(root: &Node, bin_filepath: &Path) -> Result<(), anyhow::Error> {
        let data = bincode::serde::encode_to_vec(root, bincode::config::standard())?;
        Ok(fs::write(&bin_filepath, data)?)
    }

    fn read_root_encoded(bin_filepath: &Path) -> Result<Node, anyhow::Error> {
        let data = fs::read(&bin_filepath)?;
        let node: Node = bincode::serde::decode_borrowed_from_slice(
                &data.as_slice(), bincode::config::standard()
        )?;

        Ok(node)
    }

    pub fn load(dir: PathBuf, git_repo: &git2::Repository) -> Result<Tree, anyhow::Error> {
        let root_node = Self::read_root_encoded(&path!(&dir,TREE_BIN_RELPATH))?;

        Ok(Tree {
            root: root_node,
            basedir: dir,
        })
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use lazy_static::lazy_static;
    use crate::{testing::Testing, schema::tree::NodeTrait};
    use super::{path};
    use crate::schema;
    use crate::schema::tree;
    use std::path::{Path, PathBuf};

    const FIXTURE_DIR: &'static str = "tests/fixtures/schema-tree/zulucorp";
    const FIXTURE_TREE_DUMP_FILE: &'static str = "tests/fixtures/schema-tree/zulucorp_tree_root_dump.txt";
    const FIXTURE_TREE_DATA_FILE: &'static str = "tests/fixtures/schema-tree/zulucorp_tree_root.dat";

    lazy_static! {
        static ref TESTING: Testing = {
            //shutdown_hooks::add_shutdown_hook(teardown);
            Testing::setup("/tmp/dirz-lib-test/schema/tree")
        };
    }

    extern fn teardown() {
        TESTING.teardown();
    }

    // names
    const ZULUCORP_NAME:&'static str = "zulucorp";
    const ACCOUNTING_NAME:&'static str = "accounting";
    const SOURCES_NAME:&'static str = "sources";
    const TRANSACTIONS_NAME:&'static str = "transactions";
    // labels
    const ZULUCORP_LABEL: &'static str = "Zulu Corporation";
    const ACCOUNTING_LABEL: &'static str = "Accounting Department";
    const SOURCES_LABEL: &'static str = "Source Documents";
    const TRANSACTIONS_LABEL: &'static str = "Transactions";

    #[test]
    fn schema_tree_zulucorp_traversal() {
        let dir = path!(FIXTURE_DIR);
        let schema_tree = schema::Tree::read(&dir).unwrap();

        #[cfg(debug_assertions)] {
            println!("{:#?}", schema_tree);
        }

        fn assert_node(node: &tree::Node, name: &str, label: &str, node_type: tree::NodeType, num_children: usize, has_parent: bool) {
            assert_eq!(name, node.schema().name);
            assert_eq!(label, node.schema().label);
            assert_eq!(node_type, node.node_type(), "Unexpected node type: {}", name);
            assert_eq!(num_children, node.children().len(), "Unexpected number of children: {}", name);
            assert_eq!(has_parent, node.has_parent(), "Unexpected parent or lack of parent: {}", name);
        }

        // Test 1. Verify downards traversal: root -> transactions. Cherry-pick field validation during process.

        // T 1.1 validate root
        let node = &schema_tree.root; // zulucorp
        assert_node(&node, ZULUCORP_NAME, ZULUCORP_LABEL, tree::NodeType::Path, 1, false);

        // T 1.2 validate root (zulucorp) / pathnode (accounting)
        let node = node.child(ACCOUNTING_NAME).unwrap(); // zulucorp/accounting
        assert_node(&node, ACCOUNTING_NAME, ACCOUNTING_LABEL, tree::NodeType::Path, 1, true);

        // T 1.3 validate root (zulucorp) / pathnode (accounting) / pathnode (sources)
        let node = node.child(SOURCES_NAME).unwrap(); // zulucorp/accounting/sources
        assert_node(&node, SOURCES_NAME, SOURCES_LABEL, tree::NodeType::Path, 1, true);

        // T 1.4 validate root (zulucorp) / pathnode (accounting) / pathnode (sources) / shardedpathnode (transactions)
        let node = node.child(TRANSACTIONS_NAME).unwrap(); // zulucorp/accounting/sources/::/transactions
        assert_node(&node, TRANSACTIONS_NAME, TRANSACTIONS_LABEL, tree::NodeType::ShardedPath, 0, true);

        // Test 2. Verify upwards traversal: transactions -> root. Verify that root (zulucorp) is the expected node
        let node = node  // zulucorp/accounting/sources/::/transactions ...
            .parent(&schema_tree).unwrap()  // zulucorp/accounting/sources ...
            .parent(&schema_tree).unwrap()  // zulucorp/accounting ...
            .parent(&schema_tree).unwrap();  // zulucorp

        assert_eq!(ZULUCORP_NAME, node.schema().name);
        assert!(node.has_child(ACCOUNTING_NAME), "Expected to have a specific child");

        let node = schema_tree.child_for(path!(ACCOUNTING_NAME, SOURCES_NAME, TRANSACTIONS_NAME).to_str().unwrap())
            .unwrap();
        let txn_node = node;
        assert_eq!(TRANSACTIONS_NAME, txn_node.schema().name);

        // T 1.1 Expect that each record field exists
        let record_schema = txn_node.schema().record_schema.as_ref().unwrap();
        const TXN_RECORD_FIELD_NAMES: [&'static str;6] = [ "ident", "order", "party", "date", "account", "summary" ];
        assert_eq!(TXN_RECORD_FIELD_NAMES.len(), record_schema.fields.len());

        for field_name in TXN_RECORD_FIELD_NAMES {
            assert!(record_schema.fields.contains_key(field_name));
        }

    }

    #[test]
    fn readdump_schema_tree_zulucorp() {
        let dir = path!(FIXTURE_DIR);
        let schema_tree = schema::Tree::read(&dir).unwrap();
        let expected_dump = fs::read_to_string(path!(FIXTURE_TREE_DUMP_FILE)).unwrap();
        let actual_dump = format!("{:#?}", schema_tree.root);
        assert_eq!(expected_dump, actual_dump);
    }

    #[test]
    fn tree_binary_io_and_eq() {
        let dir = path!(FIXTURE_DIR).canonicalize().unwrap();
        let bin_filepath = path!(TESTING.tmp_dir(), tree::TREE_BIN_FILENAME);

        // write to tmp
        let schema_tree_out = schema::Tree::read(&dir).unwrap();
        schema::Tree::write_root_encoded(&schema_tree_out.root, &bin_filepath).unwrap();

        // read back from tmp
        let schema_tree_in = schema::Tree {
            root: schema::Tree::read_root_encoded(&bin_filepath).unwrap(),
            basedir: dir
        };

        // compare using Debug/Display and using PartialEq
        let dump_out = format!("{:#?}", schema_tree_out);
        let dump_in = format!("{:#?}", schema_tree_in);
        assert_eq!(dump_out, dump_in, "Schema tree Debug/Display does not match after read/write");
        assert_eq!(schema_tree_out, schema_tree_in, "Schema tree PartialEq does not match after read/write");
    }

    #[test]
    fn tree_save_load() {
        let dir = TESTING.create_subtest_dir_copy(&path!(FIXTURE_DIR), "tree_save_load");
        let schema_tree_out = schema::Tree::read(&dir).unwrap();

        // save
        schema_tree_out.save().unwrap();

        // load back from save
        let git_repo = init_git(&dir);
        let schema_tree_in = schema::Tree::load(dir, &git_repo).unwrap();

        assert_eq!(schema_tree_out, schema_tree_in, "Schema tree PartialEq does not match after read/write");
    }

    fn init_git(dir: &Path) -> git2::Repository {
        let repo = git2::Repository::init(&dir).unwrap();
        let paths: Vec<PathBuf> = vec![  // todo: list dir
            PathBuf::from(".cabinet/schema.toml"),
        ];

        git_init_commit(&repo, &path!(".gitignore")).unwrap();
        git_add_and_commit(&repo, "added schema", paths).unwrap();

        repo
    }

    fn git_find_last_commit(repo: &git2::Repository) -> Result<git2::Commit, git2::Error> {
        let obj = repo.head()?.resolve()?.peel(git2::ObjectType::Commit)?;
        obj.into_commit().map_err(|_| git2::Error::from_str("Couldn't find commit"))
    }

    fn git_init_commit(repo: &git2::Repository, path: &Path) -> Result<git2::Oid, git2::Error> {
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

    fn git_add_and_commit(repo: &git2::Repository, message: &str, paths: Vec<PathBuf>) -> Result<git2::Oid, git2::Error> {
        let mut index = repo.index()?;  // retrieve the index file for the repository

        for path in paths {
            if !repo.is_path_ignored(&path).unwrap() {
                index.add_path(&path)?;  // doesn't follow .gitignore rules. path is relative to repo working dir.
            }
        }

        index.write().unwrap();
        let oid = index.write_tree()?;  // recursively writes an index
        let signature = git2::Signature::now("Testing", "testing@localhost")?;
        let parent_commit = git_find_last_commit(&repo)?;
        let tree = repo.find_tree(oid)?;
        repo.commit(Some("HEAD"), //  point HEAD to our new commit
                    &signature, // author
                    &signature, // committer
                    message, // commit message
                    &tree, // tree
                    &[&parent_commit])
    }

    #[test]
    #[ignore]
    fn writedump_zulucorp_tree_fixture() {
        let dir = path!(FIXTURE_DIR);
        let schema_tree = schema::Tree::read(&dir).unwrap();
        let expected_dump = format!("{:#?}", schema_tree.root);
        fs::write(path!(FIXTURE_TREE_DUMP_FILE), expected_dump).expect("Unable to write fixture dump");
    }

}

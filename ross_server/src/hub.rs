use rocksdb;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;

const PROJECTS_CF: &str = "projects";
const LIVE_CHANGES_CF: &str = "hot";
const BRANCHES_CF: &str = "branches";
const COMMITS_CF: &str = "commits";

pub type CommitHash = String;
pub type ObjectUUID = String;
pub type Timestamp = u64;
pub type Author = String;

#[derive(Serialize, Deserialize)]
pub struct BranchMeta {
    pub created: Timestamp,
    pub branch_start: CommitHash,
    pub head: CommitHash,
}

#[derive(Serialize, Deserialize)]
pub struct Commit {
    pub date: String,
    pub authors: Vec<String>,
    pub patch: Vec<Action>,
}

#[derive(Serialize, Deserialize)]
pub enum Action {
    Create(CreateAction),
    CAS(CASAction),
    Delete(ObjectUUID),
}

#[derive(Serialize, Deserialize)]
pub struct CreateAction {
    uuid: String,
    data: Object,
}

#[derive(Serialize, Deserialize)]
pub struct CASAction {
    object: ObjectUUID,
    field: String,
    current: Value,
    next: Value,
}

#[derive(Serialize, Deserialize)]
pub struct Object {
    data: HashMap<String, Value>,
}

#[derive(Serialize, Deserialize, PartialEq)]
pub enum Value {
    Null,
    True,
    False,
    String(String),
    Number(f64),
}

/// Hub is the central database of this software, it owns projects,
/// and there is most probably only one hub in each project, we define
/// it in order to have multiple projects on the same rocksdb instance.
pub struct Hub {
    /// We use the RocksDB as our underlying database to store each project's
    /// information, overall we need a key-value store that stores pairs of
    /// key-value, in this software our key-value pairs fit one of these
    /// categories:
    /// 1. Project -> Branches
    /// 2. Branch -> Live Changes
    /// 3. Branch -> Meta (Head Commit, Creation Date, ...)
    /// 4. Commit -> Meta (Author(s), Committer, Date, Parent), Patch
    /// We use a separate column family to store each of these stuff.
    db: rocksdb::DB,
}

pub struct Session<'a> {
    hub: &'a Hub,
    branch: String,
    hot: Vec<(Timestamp, Author, Action)>,
    data: HashMap<ObjectUUID, Object>,
}

impl Hub {
    pub fn open(path: &str) -> Result<Self, rocksdb::Error> {
        let mut opts = rocksdb::Options::default();
        opts.create_if_missing(true);

        let db = rocksdb::DB::open_cf(
            &opts,
            path,
            vec![PROJECTS_CF, LIVE_CHANGES_CF, BRANCHES_CF, COMMITS_CF],
        )?;

        Ok(Hub { db })
    }

    pub fn get_commit(&self, hash: CommitHash) -> Result<Option<Commit>, Box<dyn Error>> {
        let cf = self.db.cf_handle(COMMITS_CF).unwrap();
        match self.db.get_pinned_cf(cf, hash)? {
            Some(serialized) => Ok(Some(bincode::deserialize::<Commit>(serialized.as_ref())?)),
            None => Ok(None),
        }
    }

    pub fn put_commit(&self, hash: CommitHash, commit: Commit) -> Result<(), Box<dyn Error>> {
        let serialized = bincode::serialize(&commit)?;
        let cf = self.db.cf_handle(COMMITS_CF).unwrap();
        self.db.put_cf(cf, hash, serialized)?;
        Ok(())
    }

    pub fn get_branch_meta(&self, id: String) -> Result<Option<BranchMeta>, Box<dyn Error>> {
        let cf = self.db.cf_handle(BRANCHES_CF).unwrap();
        match self.db.get_pinned_cf(cf, id)? {
            Some(serialized) => Ok(Some(bincode::deserialize::<BranchMeta>(
                serialized.as_ref(),
            )?)),
            None => Ok(None),
        }
    }

    pub fn put_branch_meta(&self, id: String, meta: BranchMeta) -> Result<(), Box<dyn Error>> {
        let serialized = bincode::serialize(&meta)?;
        let cf = self.db.cf_handle(BRANCHES_CF).unwrap();
        self.db.put_cf(cf, id, serialized)?;
        Ok(())
    }
}

impl<'hub> Session<'hub> {
    pub fn open(hub: &'hub Hub, branch: String) -> Self {
        Session {
            hub,
            branch,
            hot: Vec::with_capacity(32),
            data: HashMap::new(),
        }
    }

    pub fn commit(&mut self, msg: String) {}

    pub fn revert(&mut self) {}

    pub fn perform(&mut self, action: Action) {}
}

use crate::action::Transaction;
use crate::hash::{Hash16, Hash20};
use crate::{Timestamp, UserID};
use serde::{Deserialize, Serialize};
use sha1::{Digest, Sha1};
use std::fmt::Write;

#[derive(Debug, Serialize, Deserialize, Ord, PartialOrd, Eq, PartialEq, Hash, Clone, Copy)]
pub struct CommitIdentifier {
    pub repository: Hash16,
    pub hash: Hash20,
}

/// The commit info is used to store information regarding a commit, it is
/// store in the database, so any change to this struct is considered a breaking
/// change.
#[derive(Debug, Serialize, Deserialize)]
pub struct CommitInfo {
    /// The branch in which this commit took place the first time.
    pub branch: Hash16,
    /// Parents of this commit, usually each commit has only one parent, which is the
    /// previous commit, the initial commit has no parents, merge commits have 2 parents
    /// or even more.
    pub parents: Vec<Hash20>,
    /// When the commit was created.
    pub time: Timestamp,
    /// List of all the authors of the commit.
    pub authors: Vec<UserID>,
    /// The peron who committed the changes.
    pub committer: UserID,
    /// The commit message.
    pub message: String,
    /// The diff relative to the parent.
    pub actions: Vec<Transaction>,
}

impl CommitInfo {
    /// Create the initial commit.
    pub fn init(branch: Hash16, uid: UserID) -> Self {
        CommitInfo {
            branch,
            parents: Vec::new(),
            time: crate::now(),
            authors: Vec::new(),
            committer: uid,
            message: String::from("Init"),
            actions: Vec::new(),
        }
    }

    /// Return a textual representation of the commit, this method is somewhat
    /// equivalent to `git cat-file commit %commit` and is used to generate the
    /// hash of the commit.
    pub fn text(&self) -> String {
        let mut result = String::with_capacity(256);
        write!(&mut result, "branch {}\n", String::from(&self.branch)).unwrap();
        for parent in &self.parents {
            write!(&mut result, "parent {}\n", String::from(parent)).unwrap();
        }
        for author in &self.authors {
            write!(&mut result, "author {}\n", String::from(author)).unwrap();
        }
        write!(&mut result, "timestamp {}\n", self.time).unwrap();
        write!(
            &mut result,
            "committed-by {}\n\n",
            String::from(&self.committer)
        )
        .unwrap();
        result.push_str(&self.message);
        result
    }

    /// Generate the hash of the commit.
    pub fn hash(&self) -> Hash20 {
        let text = self.text();
        let data = format!("commit {}{}\0", text.len(), text);
        let mut hasher = Sha1::new();
        hasher.update(data);
        let slice: [u8; 20] = hasher.finalize().into();
        Hash20::from(slice)
    }
}

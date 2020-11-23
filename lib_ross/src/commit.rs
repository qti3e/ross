use crate::action::Transaction;
use crate::hash::{Hash16, Hash20};
use crate::{Timestamp, UserID};
use serde::{Deserialize, Serialize};
use sha1::{Digest, Sha1};
use std::fmt::Write;

#[derive(Debug, Serialize, Deserialize, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct CommitIdentifier {
    pub project: Hash16,
    pub hash: Hash20,
}

/// The commit info is used to store information regarding a commit, it is
/// store in the database, so any change to this struct is considered a breaking
/// change.
#[derive(Debug, Serialize, Deserialize)]
pub struct CommitInfo {
    /// The branch in which this commit took place the first time.
    branch: Hash16,
    /// Parents of this commit, usually each commit has only one parent, which is the
    /// previous commit, the initial commit has no parents, merge commits have 2 parents
    /// or even more.
    parents: Vec<Hash20>,
    /// When the commit was created.
    time: Timestamp,
    /// List of all the authors of the commit.
    authors: Vec<UserID>,
    /// The peron who committed the changes.
    committer: UserID,
    /// The commit message.
    message: String,
    /// The diff relative to the parent.
    actions: Vec<Transaction>,
}

impl CommitInfo {
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

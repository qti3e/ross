use crate::action::Transaction;
use crate::hash::{Hash16, Hash20};
use crate::{Timestamp, UserID};
use sha1::{Sha1, Digest};
use serde::{Deserialize, Serialize};

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
    /// Parent of this commit.
    parent: Option<Hash20>,
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
        match &self.parent {
            Some(parent) => format!(
                "tree {}\nparent {}\nauthors {:?}\ncommitter {}\ntimestamp {}\n\n{}",
                String::from(&self.branch),
                String::from(parent),
                self.authors.iter().map(|a| String::from(a)).collect::<Vec<String>>(),
                String::from(&self.committer),
                self.time,
                self.message
            ),
            None => format!(
                "tree {}\nauthors {:?}\ncommitter {}\ntimestamp {}\n\n{}",
                String::from(&self.branch),
                self.authors.iter().map(|a| String::from(a)).collect::<Vec<String>>(),
                String::from(&self.committer),
                self.time,
                self.message
            ),
        }
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

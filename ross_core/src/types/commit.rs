use crate::prelude::*;
use serde::{Deserialize, Serialize};
use sha1::{Digest, Sha1};
use std::fmt::Write;

/// Hash of the commit, which is SHA-1 of commit data.
pub type CommitHash = Hash20;

/// CommitHash prefixed with repository id.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub struct CommitIdentifier {
    pub repository: RepositoryId,
    pub hash: CommitHash,
}

/// The beginning part of the `CommitInfo` struct, a lot of times, we only
/// want to get this information about a commit, so it's only fair if we group
/// this info together, put it at the beginning of `CommitInfo` and then when
/// we request this info we can
#[derive(Debug, Serialize, Deserialize)]
pub struct CommitOriginInfo {
    pub branch: BranchIdentifier,
    pub fork_point: ForkPoint,
    /// An incremental numeric value that helps us decide if a commit is an
    /// ancestor of another commit or not, this number is localized on each
    /// branch, so we have `o1.branch == o2.branch`, comparing `order`s will
    /// tell us whether `o1` precedes `o2` or not, but when
    /// `o1.branch != o2.branch`, this number will not tell us anything.
    pub order: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CommitInfo {
    // It might seem logical to use `#[serde(flatten)]` here, but it won't work and
    // panics with `bincode`.
    // https://github.com/servo/bincode/issues/245)
    pub origin: CommitOriginInfo,
    pub time: Timestamp,
    pub parents: Vec<CommitIdentifier>,
    pub committer: UserId,
    pub authors: Vec<UserId>,
    pub message: String,
}

impl CommitInfo {
    /// Return a textual representation of the commit, this method is somewhat
    /// equivalent to `git cat-file commit %commit` and is used to generate the
    /// hash of the commit.
    #[inline]
    pub fn text(&self) -> String {
        let mut result = String::with_capacity(512);
        write!(
            &mut result,
            "branch {}\n",
            String::from(&self.origin.branch.uuid)
        )
        .unwrap();
        if let Some((_, commit)) = &self.origin.fork_point {
            write!(&mut result, "tree {}\n", String::from(&commit.hash)).unwrap();
        }
        for parent in &self.parents {
            write!(&mut result, "parent {}\n", String::from(&parent.hash)).unwrap();
        }
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
    pub fn hash(&self) -> CommitHash {
        let text = self.text();
        let data = format!("commit {}{}\0", text.len(), text);
        let mut hasher = Sha1::new();
        hasher.update(data);
        let slice: [u8; 20] = hasher.finalize().into();
        CommitHash::from(slice)
    }
}

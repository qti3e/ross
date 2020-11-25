use crate::*;
use db::*;
use serde::{Deserialize, Serialize};
use sha1::{Digest, Sha1};
use std::fmt::Write;

#[derive(Debug, Serialize, Deserialize, Ord, PartialOrd, Eq, PartialEq, Hash, Clone, Copy)]
pub struct CommitIdentifier {
    pub repository: RepositoryID,
    pub hash: CommitID,
}

/// The commit info is used to store information regarding a commit, it is
/// store in the database, so any change to this struct is considered a breaking
/// change.
#[derive(Debug, Serialize, Deserialize)]
pub struct CommitInfo {
    /// The branch in which this commit took place the first time.
    pub branch: branch::BranchIdentifier,
    /// Alias for `self.branch.fork_point` to reduce number of db reads when trying to
    /// find the LCA.
    pub fork_point: Option<(branch::BranchIdentifier, CommitIdentifier)>,
    /// Parents of this commit, usually each commit has only one parent, which is the
    /// previous commit, the initial commit has no parents, merge commits have 2 parents
    /// or even more.
    /// We store `CommitIdentifier`, which would allow us to have cross repository forks.
    pub parents: Vec<CommitIdentifier>,
    /// When the commit was created.
    pub time: Timestamp,
    /// The peron who committed the changes.
    pub committer: UserID,
    /// The commit message.
    pub message: String,
    /// The diff relative to the parent.
    pub actions: Vec<action::Transaction>,
}

impl CommitInfo {
    /// Create the initial commit.
    pub fn init(branch: branch::BranchIdentifier, uid: UserID) -> Self {
        CommitInfo {
            branch,
            fork_point: None,
            parents: Vec::new(),
            time: crate::now(),
            committer: uid,
            message: String::from("Init"),
            actions: Vec::new(),
        }
    }

    /// Return a textual representation of the commit, this method is somewhat
    /// equivalent to `git cat-file commit %commit` and is used to generate the
    /// hash of the commit.
    pub fn text(&self) -> String {
        let mut result = String::with_capacity(512);
        write!(&mut result, "branch {}\n", String::from(&self.branch.uuid)).unwrap();
        if let Some((_, commit)) = &self.fork_point {
            write!(&mut result, "tree {}\n", String::from(&commit.hash)).unwrap();
        }
        for parent in &self.parents {
            write!(&mut result, "parent {}\n", String::from(&parent.hash)).unwrap();
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
    pub fn hash(&self) -> CommitID {
        let text = self.text();
        let data = format!("commit {}{}\0", text.len(), text);
        let mut hasher = Sha1::new();
        hasher.update(data);
        let slice: [u8; 20] = hasher.finalize().into();
        CommitID::from(slice)
    }

    /// Add the required db operations for this commit to happen into a database
    /// batch write.
    pub fn commit(
        &self,
        batch: &mut Batch,
        repository: RepositoryID,
        snapshot: &snapshot::Snapshot,
    ) -> CommitIdentifier {
        let hash = self.hash();
        let id = CommitIdentifier { repository, hash };

        // 1. Log the event.
        batch.push(
            keys::Log(repository),
            &log::LogEvent::Commit {
                time: self.time,
                uid: self.committer,
                branch: self.branch.uuid,
                hash,
            },
        );
        // 2. Store the commit.
        batch.put(keys::CommitInfo(id), self);
        // 3. Store the snapshot.
        batch.put(keys::Snapshot(id), snapshot);

        id
    }
}

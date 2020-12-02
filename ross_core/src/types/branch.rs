use crate::db::{keys, Batch};
use crate::prelude::*;
use md5;
use serde::{Deserialize, Serialize};
use std::fmt::Write;

/// The unique identifier of a branch.
pub type BranchId = Hash16;

/// Branch identifier is used to limit the key space of branches in
/// a repository.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub struct BranchIdentifier {
    pub repository: RepositoryId,
    pub uuid: BranchId,
}

/// The point in which this branch was forked form.
/// ```text
/// - C0 - C1 - C2 ---------> Main
///         \_______ C3 ----> Forked
/// ```
/// For example in the above case the `fork_point` of branch `Forked` is
/// `(Branch(Main), Commit(C1))`.
///
/// TODO(qti3e) Consider investigating this edge-case.
/// ```text
/// - C0 - C1 - C2 -------------> Main
///         \_______ C3 -------> Forked
///         \____________C4 ---> Fork 2
/// `Fork 2` is forked from `Forked` and not `Main`.
/// LCA(C4, C3) = LCA(C4, C2) = C1
/// ```
pub type ForkPoint = Option<(BranchIdentifier, CommitIdentifier)>;

/// The information regarding a branch, this struct is stored in the DB.
#[derive(Debug, Serialize, Deserialize)]
pub struct BranchInfo {
    /// Current head of the commit.
    pub head: CommitIdentifier,
    /// If `head` is the last commit on a branch, `fork_point` is the tail.  
    /// See [ForkPoint](ForkPoint).
    pub fork_point: ForkPoint,
    /// When was this branch created.
    pub created_at: Timestamp,
    /// The user who created the branch.
    pub user: UserId,
    /// An static branch is a branch that cannot have live-changes and can therefore
    /// only be updated using merges.  
    /// As a usage we can point to `production` branch in a repository.
    pub is_static: bool,
    /// An archived branch is a read-only branch, no further changes are allowed.
    pub is_archived: bool,
    /// The human-readable title of the branch for display-only purposes.
    pub title: String,
}

impl BranchInfo {
    #[inline]
    pub fn text(&self) -> String {
        let mut result = String::with_capacity(512);
        if let Some((branch, commit)) = &self.fork_point {
            write!(
                &mut result,
                "fork-point: {}@{}\n",
                String::from(&commit.hash),
                String::from(&branch.uuid)
            )
            .unwrap();
        }
        write!(&mut result, "created-at: {}\n", self.created_at).unwrap();
        write!(&mut result, "user: {}\n\n", String::from(&self.user)).unwrap();
        write!(&mut result, "{}", self.title).unwrap();
        result
    }

    pub fn hash(&self) -> BranchId {
        let text = self.text();
        let data = format!("branch {}{}\0", text.len(), text);
        let slice: [u8; 16] = md5::compute(data).into();
        BranchId::from(slice)
    }

    pub fn write_branch(
        &self,
        batch: &mut Batch,
        repository: RepositoryId,
        uuid: Option<BranchId>,
    ) -> BranchIdentifier {
        let uuid = uuid.unwrap_or_else(|| self.hash());
        let id = BranchIdentifier { repository, uuid };
        // 1. Log the event.
        // 2. Write info.
        // 3. Add branch to repository.
        batch.push(
            keys::Log(repository),
            &LogEvent::BranchCreated {
                user: self.user,
                id: uuid,
                head: self.head.hash,
                time: self.created_at,
            },
        );
        batch.put(keys::Branch(id), &self);
        batch.push(keys::Branches(repository), &uuid);
        id
    }

    /// Create an initial `main` branch, the head needs to be set later.
    pub fn init(time: Timestamp, user: UserId) -> Self {
        BranchInfo {
            head: CommitIdentifier::default(),
            fork_point: None,
            created_at: time,
            user,
            is_static: true,
            is_archived: false,
            title: "Main".into(),
        }
    }
}

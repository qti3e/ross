use crate::db::*;
use crate::*;
use std::collections::BTreeMap;

/// Find the lowest common ancestor between a set of commits, the LCA can be
/// used as the merge-base.
pub fn lca<'a>(
    db: &'a DBSync,
    commits: Vec<commit::CommitIdentifier>,
) -> error::Result<commit::CommitIdentifier> {
    if commits.len() == 2 {
        lca2(db, commits[0], commits[1])
    } else {
        // TODO(qti3e) Implement LCA for more than 2 commits.
        Err(error::Error::LcaNotFound)
    }
}

fn lca2<'a>(
    db: &'a DBSync,
    a: commit::CommitIdentifier,
    b: commit::CommitIdentifier,
) -> error::Result<commit::CommitIdentifier> {
    let db = db.read()?;
    let mut seen =
        BTreeMap::<branch::BranchIdentifier, Option<(commit::CommitIdentifier, Timestamp)>>::new();

    let commit_a = db
        .get(keys::CommitInfo(a))?
        .ok_or_else(|| error::Error::CommitNotFound)?;
    let commit_b = db
        .get(keys::CommitInfo(b))?
        .ok_or_else(|| error::Error::CommitNotFound)?;
    seen.insert(commit_a.branch, None);
    seen.insert(commit_b.branch, None);

    let mut q_slice: [Option<commit::CommitIdentifier>; 2] = [None; 2];
    let mut q = fixed_queue::FixedSizeQueue::new(&mut q_slice);

    if let Some((branch, id)) = commit_b.fork_point {
        match seen.insert(branch, Some((id, commit_b.time))) {
            Some(_) => {
                return Ok(id);
            }
            _ => {}
        }
        q.enqueue(id);
    }

    if let Some((branch, id)) = commit_a.fork_point {
        match seen.insert(branch, Some((id, commit_a.time))) {
            Some(_) => {
                return Ok(id);
            }
            _ => {}
        }
        q.enqueue(id);
    }

    while !q.is_empty() {
        let id = q.dequeue();
        let commit = db
            .get(keys::CommitInfo(id))?
            .ok_or_else(|| error::Error::CommitNotFound)?;
        if commit.fork_point.is_none() {
            continue;
        }
        let (branch, commit_id) = commit.fork_point.unwrap();
        match seen.insert(branch, Some((commit_id, commit.time))) {
            Some(None) => {
                return Ok(id);
            }
            Some(Some((other, time))) if time < commit.time => {
                return Ok(other);
            }
            Some(Some(_)) => {
                return Ok(id);
            }
            None => {}
        }
        q.enqueue(commit_id);
    }

    Err(error::Error::LcaNotFound)
}

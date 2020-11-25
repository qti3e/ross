use crate::*;
use std::collections::BTreeMap;

pub struct LcaCommitData {
    time: Timestamp,
    branch: branch::BranchIdentifier,
    fork_point: Option<(branch::BranchIdentifier, commit::CommitIdentifier)>,
}

/// Find the lowest common ancestor between a set of commits, the LCA can be
/// used as the merge-base.
pub fn lca<'a>(
    get_commit_fn: &mut impl FnMut(&commit::CommitIdentifier) -> error::Result<&'a LcaCommitData>,
    commits: Vec<commit::CommitIdentifier>,
) -> error::Result<commit::CommitIdentifier> {
    if commits.len() == 2 {
        lca2(get_commit_fn, &commits[0], &commits[1])
    } else {
        // TODO(qti3e) Implement LCA for more than 2 commits.
        Err(error::Error::LcaNotFound)
    }
}

fn lca2<'a>(
    get_commit_fn: &mut impl FnMut(&commit::CommitIdentifier) -> error::Result<&'a LcaCommitData>,
    a: &commit::CommitIdentifier,
    b: &commit::CommitIdentifier,
) -> error::Result<commit::CommitIdentifier> {
    let mut seen =
        BTreeMap::<branch::BranchIdentifier, Option<(commit::CommitIdentifier, Timestamp)>>::new();

    let commit_a = get_commit_fn(a)?;
    let commit_b = get_commit_fn(b)?;
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
        let commit = get_commit_fn(&id)?;
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

#[cfg(test)]
mod test {
    use super::*;
    use std::collections::HashMap;

    struct BranchData {
        fork_point: Option<(branch::BranchIdentifier, commit::CommitIdentifier)>,
        head: Option<commit::CommitIdentifier>,
    }

    struct Graph {
        time: Timestamp,
        commits: HashMap<commit::CommitIdentifier, LcaCommitData>,
        branches: HashMap<branch::BranchIdentifier, BranchData>,
    }

    impl Graph {
        pub fn new() -> Self {
            Self {
                time: 0,
                commits: HashMap::new(),
                branches: HashMap::new(),
            }
        }

        pub fn init(&mut self) -> branch::BranchIdentifier {
            let id = branch::BranchIdentifier {
                repository: RepositoryID::MIN,
                uuid: BranchID::rand(),
            };

            self.branches.insert(
                id,
                BranchData {
                    fork_point: None,
                    head: None,
                },
            );

            id
        }

        pub fn commit(&mut self, branch: &branch::BranchIdentifier) -> commit::CommitIdentifier {
            let id = commit::CommitIdentifier {
                repository: RepositoryID::MIN,
                hash: CommitID::rand(),
            };

            let b = self.branches.get_mut(branch).unwrap();
            b.head = Some(id);
            let time = self.time;
            self.time += 1;
            self.commits.insert(
                id,
                LcaCommitData {
                    time,
                    branch: *branch,
                    fork_point: b.fork_point,
                },
            );

            id
        }

        pub fn fork(&mut self, branch: &branch::BranchIdentifier) -> branch::BranchIdentifier {
            let id = branch::BranchIdentifier {
                repository: RepositoryID::MIN,
                uuid: BranchID::rand(),
            };

            let b = self.branches.get(branch).unwrap();
            let head = b.head;
            self.branches.insert(
                id,
                BranchData {
                    fork_point: Some((*branch, head.unwrap())),
                    head,
                },
            );

            id
        }

        pub fn get_commit(&self, id: &commit::CommitIdentifier) -> error::Result<&LcaCommitData> {
            self.commits
                .get(id)
                .ok_or_else(|| error::Error::CommitNotFound)
        }
    }

    #[test]
    fn basic() {
        // - * - M ---- * ------ A --> B0
        //       \
        //        -- * ---- B -------> B1
        let mut g = Graph::new();
        let b0 = g.init();

        g.commit(&b0);
        let m = g.commit(&b0);
        let b1 = g.fork(&b0);
        g.commit(&b1);
        g.commit(&b0);
        let b = g.commit(&b1);
        let a = g.commit(&b0);

        let mut count = 0;
        let mut get = |k: &commit::CommitIdentifier| {
            count += 1;
            g.get_commit(k)
        };
        assert_eq!(lca(&mut get, vec![a, b]).unwrap(), m);
        assert_eq!(count, 2);
    }
}

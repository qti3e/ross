use std::collections::HashMap;
use crate::types::*;
use crate::error;
use super::ring_buffer::RingBuffer;

/// Find the lowest common ancestor between a set of commits, the LCA can be
/// used as the merge-base.
pub fn lca<'a>(
    get_commit_fn: &mut impl FnMut(&CommitIdentifier) -> error::Result<&'a CommitInfoOrigin>,
    commits: Vec<CommitIdentifier>,
) -> error::Result<CommitIdentifier> {
    if commits.len() == 2 {
        lca2(get_commit_fn, &commits[0], &commits[1])
    } else {
        // TODO(qti3e) Implement LCA for more than 2 commits.
        Err(error::Error::LcaNotFound)
    }
}

#[inline]
fn lca2<'a>(
    get_commit_fn: &mut impl FnMut(&CommitIdentifier) -> error::Result<&'a CommitInfoOrigin>,
    a: &CommitIdentifier,
    b: &CommitIdentifier,
) -> error::Result<CommitIdentifier> {
    let mut commit_a = get_commit_fn(a)?;
    let mut commit_b = get_commit_fn(b)?;
    let min = if commit_b.order > commit_a.order {
        std::mem::swap(&mut commit_a, &mut commit_b);
        a
    } else {
        b
    };

    if commit_a.branch == commit_b.branch {
        return Ok(*min);
    }

    let mut seen =
        HashMap::<BranchIdentifier, Option<(CommitIdentifier, u32)>>::with_capacity(8);
    seen.insert(commit_a.branch, None);
    seen.insert(commit_b.branch, None);

    let mut q_slice: [Option<CommitIdentifier>; 2] = [None; 2];
    let mut q = RingBuffer::new(&mut q_slice);

    if let Some((branch, id)) = commit_b.fork_point {
        match seen.insert(branch, Some((id, commit_b.order))) {
            Some(_) => {
                return Ok(id);
            }
            _ => {}
        }
        q.enqueue(id);
    }

    if let Some((branch, id)) = commit_a.fork_point {
        match seen.insert(branch, Some((id, commit_a.order))) {
            Some(None) => {
                return Ok(id);
            }
            Some(Some((other, order))) if order < commit_a.order => {
                return Ok(other);
            }
            Some(Some(_)) => {
                return Ok(id);
            }
            None => {}
        }
        q.enqueue(id);
    }

    while let Some(to_visit) = q.dequeue() {
        let commit = get_commit_fn(&to_visit)?;
        if commit.fork_point.is_none() {
            continue;
        }
        let (branch, id) = commit.fork_point.unwrap();
        match seen.insert(branch, Some((id, commit.order))) {
            Some(None) => {
                return Ok(id);
            }
            Some(Some((other, order))) if order < commit.order => {
                return Ok(other);
            }
            Some(Some(_)) => {
                return Ok(id);
            }
            None => {}
        }
        q.enqueue(id);
    }

    Err(error::Error::LcaNotFound)
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::utils::hash::*;
    use std::collections::HashMap;

    struct BranchData {
        fork_point: Option<(BranchIdentifier, CommitIdentifier)>,
        head: Option<CommitIdentifier>,
    }

    struct Graph {
        order: u32,
        commits: HashMap<CommitIdentifier, CommitInfoOrigin>,
        branches: HashMap<BranchIdentifier, BranchData>,
    }

    impl Graph {
        pub fn new() -> Self {
            Self {
                order: 0,
                commits: HashMap::new(),
                branches: HashMap::new(),
            }
        }

        pub fn init(&mut self) -> BranchIdentifier {
            let id = BranchIdentifier {
                repository: RepositoryId(Hash16::MIN),
                id: BranchId(rand::random())
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

        pub fn commit(&mut self, branch: &BranchIdentifier) -> CommitIdentifier {
            let id = CommitIdentifier {
                repository: RepositoryId(Hash16::MIN),
                hash: CommitHash(rand::random())
            };

            let b = self.branches.get_mut(branch).unwrap();
            b.head = Some(id);
            let order = self.order;
            self.order += 1;
            self.commits.insert(
                id,
                CommitInfoOrigin {
                    order,
                    branch: *branch,
                    fork_point: b.fork_point,
                },
            );

            id
        }

        pub fn fork(&mut self, branch: &BranchIdentifier) -> BranchIdentifier {
            let id = BranchIdentifier {
                repository: RepositoryId(Hash16::MIN),
                id: BranchId(rand::random()),
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

        pub fn get_commit(&self, id: &CommitIdentifier) -> error::Result<&CommitInfoOrigin> {
            self.commits
                .get(id)
                .ok_or_else(|| error::Error::CommitNotFound)
        }
    }

    #[test]
    fn lca_2() {
        //                     ------- D -----------> B2
        //                   /
        //        --- * --- B --- C ---- E ---------> B1
        //      /                 \
        //    /                    -------- F ------> B3
        // -- A ------------------------------ G ---> B0
        //                                      \
        //                                       ---- H -> B4
        let mut r = Graph::new();
        let b0 = r.init();

        let a = r.commit(&b0);
        let b1 = r.fork(&b0);
        r.commit(&b1);
        let b = r.commit(&b1);
        let b2 = r.fork(&b1);
        let c = r.commit(&b1);
        let d = r.commit(&b2);
        let b3 = r.fork(&b1);
        let e = r.commit(&b1);
        let f = r.commit(&b3);
        let g = r.commit(&b0);
        let b4 = r.fork(&b0);
        let h = r.commit(&b4);

        let test = |commits: Vec<CommitIdentifier>,
                    expected_count: usize,
                    result: CommitIdentifier| {
            let mut count = 0;
            let mut get = |k: &CommitIdentifier| {
                count += 1;
                r.get_commit(k)
            };
            assert_eq!(lca(&mut get, commits).unwrap(), result);
            assert_eq!(count, expected_count);
        };

        // Probably the most common case, we fork from a branch and then merge it back.
        test(vec![e, f], 2, c);
        test(vec![g, h], 2, g);
        // Distance = 1
        test(vec![d, f], 2, b);
        // Same branch
        test(vec![c, e], 2, c);
        test(vec![c, b], 2, b);
        test(vec![g, g], 2, g);
        test(vec![g, a], 2, a);
        // etc...
        test(vec![f, h], 3, a);
        test(vec![g, c], 2, a);
    }
}

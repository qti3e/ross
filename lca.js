class Repository {
  constructor(main_branch) {
    this.last_commit_id = 0;
    this.commits = {};
    this.branches = {
      [main_branch]: {
        name: main_branch,
        head: undefined,
        fork_point: undefined
      }
    };
  }

  fork(base_name, new_branch_name) {
    const base = this.branches[base_name];
    if (!base) throw new Error(`Branch ${base_name} not found.`);
    const fork_point = {
      branch: base,
      commit: base.head
    };
    this.branches[new_branch_name] = {
      name: new_branch_name,
      head: base.head,
      fork_point
    }
  }

  commit(branch_name, name) {
    const branch = this.branches[branch_name];
    if (!branch) throw new Error(`Branch ${branch_name} not found.`);
    const id = this.last_commit_id++;
    name = name || String(id);
    const commit = {
      id,
      name,
      branch: branch,
      fork_point: branch.fork_point, // alias for .branch.fork_point
      parent: branch.head
    }
    branch.head = commit;
    this.commits[name] = commit;
  }

  lca(a_name, b_name) {
    const commit_a = this.commits[a_name];
    const commit_b = this.commits[b_name];
    const branch_a = commit_a.branch;
    const branch_b = commit_b.branch;

    if (branch_a === branch_b) return min(commit_a, commit_b);
    const seen = new Map();
    seen.set(branch_a, undefined);
    seen.set(branch_b, undefined);

    const q = [commit_b, commit_a];

    while (q.length) {
      const c = q.shift();
      const fp = c.fork_point;
      console.log('visiting', c.name);
      if (!fp) continue;
      console.log(`FP(${c.name}) = (${fp.branch.name}, ${fp.commit.name})`);
      if (seen.has(fp.branch))
        return min(seen.get(fp.branch), fp.commit);
      seen.set(fp.branch, fp.commit);
      if (fp.commit) q.push(fp.commit);
    }
  }
}

function min(a, b) {
  if (!a) return b;
  if (!b) return a;
  if (a.id < b.id) {
    return a;
  } else {
    return b;
  }
}

const r = new Repository('B0');
//                     ------- D -----------> B2
//                   /
//        --- * --- B --- C ---- E ---------> B1
//      /                 \
//    /                    -------- F ------> B3
// -- A ------------------------------ G ---> B0
//                                      \
//                                       ---- H -> B4
r.commit('B0', 'A');
r.fork('B0', 'B1');
r.commit('B1');
r.commit('B1', 'B');
r.fork('B1', 'B2');
r.commit('B1', 'C');
r.commit('B2', 'D');
r.fork('B1', 'B3');
r.commit('B1', 'E');
r.commit('B3', 'F');
r.commit('B0', 'G');
r.fork('B0', 'B4');
r.commit('B4', 'H');

const lca = r.lca('G', 'H');
console.log('LCA =', lca ? lca.name : '');

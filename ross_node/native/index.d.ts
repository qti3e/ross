interface ContextOptions {
}

export class Context {
    constructor(path: string, opts?: ContextOptions);
}

export type BranchIdentifier = {
    repository: string,
    branch: string
}

export class Editor {
  constructor(ctx: Context, branch: BranchIdentifier, user: string);

  sync();
  sync_partial();
  perform();
}

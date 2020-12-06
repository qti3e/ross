/**
 * A 16-byte hash which is stored as a 32-char string in the client.
 */
export type Hash16 = string;

export type PrimitiveValue = boolean | string | number | Hash16;

/**
 * A pointer to an object that is stored on the server.
 */
export interface Ref<T> {
  /**
   * Each object that is stored on the server has a unique id which is
   * assigned to it upon insertion.
   */
  readonly id: Hash16;
  /**
   * An incremental numeric value describing the version of the object on
   * the client side, the point of syncing the client with server is to
   * have the same version of objects across parties.
   */
  readonly version: number;
  /**
   * The actual data that we're pointing towards, this data is readonly and
   * all changes to the data must take place using a patch.
   */
  readonly data: T;
}

/**
 * The core namespace contains all the common classes and types between all of
 * different clients.
 */
export namespace core {
  export class Snapshot {
    /**
     * The latest version of all the objects in the snapshot.
     */
    readonly objects: Record<Hash16, Ref<any>> = Object.create(null);
  }

  export class RawReader {
    private cursor = 0;
    constructor(readonly snapshot: Snapshot, readonly data: PrimitiveValue[]) {}
    next(): PrimitiveValue {
      return this.data[this.cursor++];
    }
  }

  export class Session {
    readonly snapshot: Snapshot;
    readonly user: Hash16;
  }
}

// --- functions used in the definition.

interface StructConstructor<T = any> {
  $: Field[];
  decode(reader: core.RawReader): T;
  new(): T;
}

type Field =
  // Primitive
  | string
  // Inline struct
  | [string, StructConstructor]
  // Ref<T>
  | [string];

function flattenPath(fields: Field[]): string[][] {
  const result = [];

  function write(path: string[], field: Field) {
    if (typeof field === "string" || field[1] === undefined) {
      result.push([...path, field]);
    } else {
      const newPath = [...path, field[0]];
      const fields = field[1].$;
      for (let i = 0, n = fields.length; i < n; ++i) write(newPath, fields[i]);
    }
  }

  for (let i = 0, n = fields.length; i < n; ++i) write([], fields[i]);

  return result;
}

function c(
  ns: any,
  id: number,
  name: string,
  fields: Field[],
  members: string[] = []
) {
  let flattenCache: string[][] | undefined;
  const r = function (...args: any[]) {
    for (let i = 0, n = fields.length; i < n; ++i) {
      const field = fields[i];
      const key = typeof field === "string" ? field : field[0];
      this[key] = args[i];
    }
    for (let i = 0, n = members.length; i < n; ++i) this[members[i]] = [];
  };
  r.prototype.getAllChildren = function () {
    return Array.prototype.concat.apply(
      [],
      members.map((m) => this[m])
    );
  };
  r.prototype.getPathFor = function (fieldId: number): string[] {
    if (!flattenCache) flattenCache = flattenPath(fields);
    return flattenCache[fieldId];
  };
  // Static members
  r.$ = fields;
  r.decode = function (reader: core.RawReader) {
    const values = [];
    for (let i = 0, n = fields.length; i < n; ++i) {
      const field = fields[i];
      if (typeof field === 'string') {
        values.push(reader.next());
      } else if (field[1] === undefined) {
        const id = reader.next();
        if (typeof id !== 'string') throw new TypeError('Expected Hash16.');
        values.push(reader.snapshot.objects[id]);
      } else {
        values.push(field[1].decode(reader));
      }
    }
    return new r(...values);
  };
  Object.defineProperty(r, 'name', { value: name });
  ns._[id] = ns[name] = r;
}

import type { Snapshot, Patch } from "./snapshot";
import {
  Field,
  ObjectRawData,
  RossStruct,
  PrimitiveValue,
  Ref,
  Hash16,
} from "./common";

// This file contains the functions used to generate the classes and other
// stuff dynamically at the runtime.

/**
 * Get a list of fields and return an array of the path to each field.
 * # Example
 * ```js
 * // struct Point2D {x: num, y: num}
 * // struct X {pos: Point2D, size: num}
 * flattenFields([['pos', ['x', 'y']], 'size']);
 * // -> [['pos', 'x'], ['pos', 'y'], ['size']]
 * ```
 * @param fields List of the fields of an struct.
 */
function flattenFields(fields: Field[]): string[][] {
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

/**
 * Generate a class to represent an struct from some descriptions.
 * @param ns The namespace object.
 * @param id Unique ID of the struct.
 * @param name Name of the struct.
 * @param fields List of fields.
 * @param members Name of object containers of this struct.
 */
export function c(
  ns: any,
  id: number,
  name: string,
  fields: Field[],
  members: string[],
  ownerField?: string
) {
  class Struct extends RossStruct {
    static flattenCache: string[][] | undefined;
    static readonly $ = fields;
    private _alreadyInOwner: boolean;

    constructor(...args: any[]) {
      super();
      if (ownerField && args[0] === null)
        Object.defineProperty(this, '_alreadyInOwner', {
          configurable: true,
          value: true
        });

      let arg = 0;
      for (let n = fields.length; arg < n; ++arg) {
        const field = fields[arg];
        const key = typeof field === "string" ? field : field[0];
        this[key] = args[arg];
      }
      for (let i = 0, n = members.length; i < n; ++i, ++arg) {
        const children = (this[members[i]] = Array.from(args[arg] || []));
        for (let i = 0, n = children.length; i < n; ++i) {
          const child = children[i];
          if (!(child instanceof RossStruct))
            throw new Error("Child must be another RossStruct");
          if (child.owner !== null)
            throw new Error("Object must not have an active owner.");
        }
      }
    }

    attach(ref: Ref<Struct>) {
      Object.defineProperty(this, '_ref', {
        configurable: true,
        value: ref
      });
      for (let i = 0, n = members.length; i < n; ++i) {
        const children = this[members[i]] as RossStruct[];
        for (let j = 0, n = children.length; j < n; ++j) {
          children[j].owner = ref;
        }
      }
      // If the object is owned insert it to the owner.
      if (ownerField && this.owner && !this._alreadyInOwner) {
        const ownerMembers = this.owner[ownerField] as RossStruct[];
        ownerMembers.push(this);
      }
    }

    detach() {
      Object.defineProperty(this, '_ref', {
        configurable: true,
        writable: false,
        value: undefined
      });
      for (let i = 0, n = members.length; i < n; ++i) {
        const children = this[members[i]] as RossStruct[];
        for (let j = 0, n = children.length; j < n; ++j) {
          children[j].owner = null;
        }
      }
      if (ownerField && this.owner) {
        const ownerMembers = this.owner[ownerField] as RossStruct[];
        const i = ownerMembers.indexOf(this);
        if (i >= 0) {
          ownerMembers.splice(i, 1);
        }
      }
    }

    getAllChildren() {
      return Array.prototype.concat.apply(
        [],
        members.map((m) => this[m])
      );
    }

    getPathFor(fieldId: number): string[] {
      if (!Struct.flattenCache) Struct.flattenCache = flattenFields(fields);
      return Struct.flattenCache[fieldId];
    }

    encode(ownerId?: Hash16, buffer?: ObjectRawData): ObjectRawData {
      if (!buffer) buffer = [id];

      for (let i = 0, n = fields.length; i < n; ++i) {
        const field = fields[i];
        if (typeof field === "string") {
          buffer.push(this[field]);
        } else if (field[1] === undefined) {
          if (ownerId && i === 0) {
            buffer.push(ownerId);
          } else {
            buffer.push(this[field[0]].id);
          }
        } else {
          this[field[0]].encode(undefined, buffer);
        }
      }

      return buffer;
    }

    static decode(snapshot: Snapshot, iter: Iterator<PrimitiveValue>): Struct {
      const values = [];
      for (let i = 0, n = fields.length; i < n; ++i) {
        const field = fields[i];
        if (typeof field === "string") {
          values.push(iter.next().value);
        } else if (field[1] === undefined) {
          const id = iter.next().value;
          if (typeof id !== "string") throw new TypeError("Expected Hash16.");
          values.push(snapshot.objects[id]);
        } else {
          values.push(field[1].decode(snapshot, iter));
        }
      }
      return new Struct(...values);
    }
  }

  Object.defineProperty(Struct, "name", { value: name });
  ns._[id] = ns[name] = Struct;
}

/**
 * Create an insert patch.
 * @param obj The object to insert.
 */
export function i(obj: RossStruct): (uuidFn: () => string) => Patch[] {
  const create = (
    uuidFn: () => string,
    patches: Patch[],
    obj: RossStruct,
    ownerId?: Hash16
  ) => {
    const id = uuidFn();

    patches.push({
      id,
      type: "create",
      data: obj.encode(ownerId),
    });

    if (obj.owner) {
      patches.push({
        type: "touch",
        id: obj.owner.id,
      });
    }

    const children = obj.getAllChildren();
    for (let i = 0, n = children.length; i < n; ++i)
      create(uuidFn, patches, children[i], id);
  };

  let called = false;
  return (uuidFn: () => string) => {
    if (called) throw new Error("This function should only get called once.");
    called = true;
    const patches: Patch[] = [];
    create(uuidFn, patches, obj);
    return patches;
  };
}

/**
 * Create the list of patches required in order to delete an object.
 * @param ref Reference to the object that should be deleted.
 */
export function d(ref: Ref<RossStruct>): Patch[] {
  const patches: Patch[] = [];
  const q: Ref<RossStruct>[] = [ref];

  const owner = ref.data.owner;
  if (owner) {
    patches.push({
      type: "touch",
      id: ref.id,
    });
  }

  for (let i = 0; i < q.length; ++i) {
    const ref = q[i];

    patches.push({
      type: "delete",
      id: ref.id,
      version: ref.version,
    });

    const children = ref.data.getAllChildren();
    for (let j = 0, n = children.length; j < n; ++j) q.push(children[j]._ref);
  }

  return patches;
}

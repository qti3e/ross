import type { Snapshot } from "./snapshot";
import { Field, ObjectRawData, RossStruct, PrimitiveValue } from "./common";

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
  members: string[] = []
) {
  class Struct extends RossStruct {
    static flattenCache: string[][] | undefined;
    static readonly $ = fields;

    constructor(...args: any[]) {
      super();
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

    encode(buffer?: ObjectRawData): ObjectRawData {
      if (!buffer) buffer = [id];

      for (let i = 0, n = fields.length; i < n; ++i) {
        const field = fields[i];
        if (typeof field === "string") {
          buffer.push(this[field]);
        } else if (field[1] === undefined) {
          buffer.push(this[field[0]]);
        } else {
          this[field[0]].encode(buffer);
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

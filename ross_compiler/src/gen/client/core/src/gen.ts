import type { Field, ObjectRawData, StructBase } from "./types";
import type { Snapshot } from "./snapshot";
import { RawReader } from "./reader";

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
  let flattenCache: string[][] | undefined;
  class Struct implements StructBase {
    static readonly $ = fields;

    constructor(...args: any[]) {
      for (let i = 0, n = fields.length; i < n; ++i) {
        const field = fields[i];
        const key = typeof field === "string" ? field : field[0];
        this[key] = args[i];
      }
      for (let i = 0, n = members.length; i < n; ++i) this[members[i]] = [];
    }

    getAllChildren() {
      return Array.prototype.concat.apply(
        [],
        members.map((m) => this[m])
      );
    }

    getPathFor(fieldId: number): string[] {
      if (!flattenCache) flattenCache = flattenFields(fields);
      return flattenCache[fieldId];
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

    static decode(reader: RawReader): Struct;
    static decode(snapshot: Snapshot, data: ObjectRawData): Struct;
    static decode() {
      const reader =
        arguments.length === 1
          ? arguments[0]
          : new RawReader(arguments[0], arguments[1]);
      const values = [];
      for (let i = 0, n = fields.length; i < n; ++i) {
        const field = fields[i];
        if (typeof field === "string") {
          values.push(reader.next());
        } else if (field[1] === undefined) {
          const id = reader.next();
          if (typeof id !== "string") throw new TypeError("Expected Hash16.");
          values.push(reader.snapshot.objects[id]);
        } else {
          values.push(field[1].decode(reader));
        }
      }
      return new Struct(...values);
    }
  }

  Object.defineProperty(Struct, "name", { value: name });
  ns._[id] = ns[name] = Struct;
}

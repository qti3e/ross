import type { Field } from './types';
import type { RawReader } from './reader';

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

export function c(
  ns: any,
  id: number,
  name: string,
  fields: Field[],
  members: string[] = []
) {
  let flattenCache: string[][] | undefined;
  class Struct {
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
      if (!flattenCache) flattenCache = flattenPath(fields);
      return flattenCache[fieldId];
    }

    static decode(reader: RawReader) {
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

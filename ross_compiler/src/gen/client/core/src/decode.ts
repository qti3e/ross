import { ObjectRawData, StructConstructor } from "./common";
import type { Snapshot } from "./snapshot";

declare const exports: { root: { _: Record<number, StructConstructor> } };

/**
 * Deserialize a tagged data-vector into a valid object.
 * @param snapshot The snapshot, used to resolve pointers to other objects.
 * @param data A tagged data-vector to deserialize.
 */
export function decode(snapshot: Snapshot, data: ObjectRawData) {
  const root = exports.root;
  const iter = data[Symbol.iterator]();
  const tag = iter.next().value;
  const constructor = root._[tag];
  if (!constructor) throw new Error("Invalid tag.");
  return constructor.decode(snapshot, iter);
}

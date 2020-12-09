import type { Snapshot } from "./snapshot";

/**
 * A 16-byte hash which is stored as a 32-char string in the client.
 */
export type Hash16 = string;

/**
 * A pointer to an object that is stored on the server.
 */
export interface Ref<T extends RossStruct> {
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
 * Any primitive value in ROSS.
 */
export type PrimitiveValue = boolean | string | number | Hash16;

/**
 * In ROSS (core), fields do not actually exists and all of the objects are
 * treated the same way, every object is stored as a vector of primitive values,
 * in other terms all of the labels are dropped upon sending to the server.
 * It is the job of Ross-Compiler to flatten each object and provide a way for
 * us to encode and decode each data-vector into/from an object instance.
 * The way we do this is to assign a unique id to each object (relative to its
 * position in the schema), and store each object as tagged vector, where the
 * tag is a numeric value which is the object-id and the reset of the vector
 * is just all of the object's data inlined.
 */
export type ObjectRawData = [tag: number, ...data: PrimitiveValue[]];

export interface StructConstructor<T extends RossStruct = RossStruct> {
  /* @internal */
  $: Field[];
  /* @internal */
  decode(snapshot: Snapshot, iterator: Iterator<PrimitiveValue>): T;
  new (): T;
}

export type Field =
  // Primitive
  | string
  // Inline struct
  | [string, StructConstructor]
  // Ref<T>
  | [string];

/**
 * Common methods on every struct.
 */
export abstract class RossStruct {
  /** @internal */
  owner?: Hash16;
  /**
   * Return the list of all the objects owned by this object.
   * @internal
   */
  abstract getAllChildren(): RossStruct[];
  /**
   * @param fieldId Index of the field when the data is flattened.
   * @internal
   */
  abstract getPathFor(fieldId: number): string[];
  /**
   * Encode this object as an array of primitive values.
   * @internal
   */
  abstract encode(): ObjectRawData;
}

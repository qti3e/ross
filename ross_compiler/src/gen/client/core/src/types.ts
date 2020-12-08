import type { RawReader } from "./reader";

/**
 * A 16-byte hash which is stored as a 32-char string in the client.
 */
export type Hash16 = string;

/**
 * A pointer to an object that is stored on the server.
 */
export interface Ref<T extends StructBase> {
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

export type PrimitiveValue = boolean | string | number | Hash16;

export type ObjectRawData = [tag: number, ...data: PrimitiveValue[]];

export interface StructConstructor<T extends StructBase = StructBase> {
  $: Field[];
  decode(reader: RawReader): T;
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
export interface StructBase {
  /**
   * Return the list of all the objects owned by this object.
   */
  getAllChildren(): StructBase[];
  /**
   * @param fieldId Index of the field when the data is flattened.
   * @internal
   */
  getPathFor(fieldId: number): string[];
  /**
   * Encode this object as an array of primitive values.
   */
  encode(): ObjectRawData;
}

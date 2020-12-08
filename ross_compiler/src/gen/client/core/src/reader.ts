import { PrimitiveValue } from "./types";
import { Snapshot } from "./snapshot";

/**
 * Just an array with a cursor.
 */
export class RawReader {
  private cursor = 0;

  constructor(readonly snapshot: Snapshot, readonly data: PrimitiveValue[]) {}

  /**
   * Return the next element from the data-array and advance the courser.
   */
  next(): PrimitiveValue {
    return this.data[this.cursor++];
  }
}

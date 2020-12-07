import { PrimitiveValue } from "./types";
import { Snapshot } from "./snapshot";

export class RawReader {
  private cursor = 0;
  constructor(readonly snapshot: Snapshot, readonly data: PrimitiveValue[]) {}
  next(): PrimitiveValue {
    return this.data[this.cursor++];
  }
}

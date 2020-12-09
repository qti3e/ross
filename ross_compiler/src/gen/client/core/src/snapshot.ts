import { Hash16, ObjectRawData, PrimitiveValue, Ref } from "./common";

export interface CreatePatch {
  type: "create";
  id: Hash16;
  data: ObjectRawData;
}

export interface DeletePatch {
  type: "delete";
  id: Hash16;
  version: number;
}

export interface CASPatch {
  type: "cas";
  id: Hash16;
  base: PrimitiveValue;
  target: PrimitiveValue;
}

export type Patch = CreatePatch | DeletePatch | CASPatch;

export interface BatchPatch {
  patches: Patch[];
  author?: Hash16;
  action: number;
  time: number;
}

export class Snapshot {
  /**
   * The latest version of all the objects in the snapshot.
   */
  readonly objects: Record<Hash16, Ref<any>> = Object.create(null);
}

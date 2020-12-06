export type Hash16 = string;
export type Hash20 = string;
export type ObjectUUID = Hash16;
export type Ref<T> = {
  readonly version: number,
  readonly id: ObjectUUID,
  readonly data: T
};

type PrimitiveType = null | boolean | number | string | Hash16;
type ObjectRawData = [instance: number, ...data: PrimitiveType[]];
interface SnapshotObjectInfo {
  version: number;
  data: ObjectRawData;
}

interface Struct {}

/**
 * Collection of all the objects.
 */
export class Snapshot {
  constructor(
    private readonly objects: Record<ObjectUUID, SnapshotObjectInfo>
  ) {}

  /**
   * Return the object with the given uuid.
   * @param uuid The uuid of the object.
   */
  getObjectRaw(uuid: ObjectUUID): ObjectRawData | null {
    const info = this.objects[uuid];
    if (info) return info.data;
    return null;
  }

  getObject(uuid: ObjectUUID): any {}
}

export class SessionConnection {}

export type Property = string | [string, Property[]];

function generateClass(
  name: string,
  properties: Property[],
  ownedMembers: string[] = []
): any {
  function constructor(...args: any[]) {
    for (let i = 0, n = properties.length; i < n; ++i) {
      const p = properties[i];
      const key = typeof p === "string" ? p : p[0];
      this[key] = args[i];
    }
    for (const key of ownedMembers) {
      this[key] = [];
    }
  }
  constructor.prototype.getAllNodes = function () {
    return [].concat(ownedMembers.map((key) => this[key]));
  };
  constructor.$ = properties;
  constructor.name = name;
  return constructor;
}

function insert(obj: Struct) {
  return {
    type: "insert",
    // raw: toRaw(obj)
  };
}

export namespace X {
  export declare class Color {
    static readonly $: Property[];
    readonly r: number;
    readonly g: number;
    readonly b: number;
    constructor(r: number, g: number, b: number);
  }

  X.Color = generateClass('Color', ['r', 'g', 'b']);

  export declare class Shape {
    static readonly $: Property[];
    readonly color: Color;
    readonly size: number;
    constructor(color: Color, size: number);
  }

  X.Shape = generateClass('Shape', [['color', X.Color.$], 'size']);

  export declare class Box {
    static readonly $: Property[];
    readonly title: string;
    /* Reference to owned members */
    readonly members: ReadonlyArray<Ref<OwnedBox>>;
    constructor(title: string);
  }

  X.Box = generateClass('Box', ['title'], ['members']);

  export declare class OwnedBox implements Struct {
    static readonly $: Property[];
    readonly owner: Ref<Box>;
    readonly color: Color;
    constructor(owner: Ref<Box>, color: Color);
  }

  X.OwnedBox = generateClass('OwnedBox', ['owner', ['color', X.Color.$]]);

  X.actions = {} as any;

  export declare namespace actions {
    export function insertColor(color: Color);
  }
  X.actions.insertColor = () => null;

    // return [
    // insert(color)
    // ];

  declare namespace T {
    export function changeR(color: Ref<Color>, r: number);
    // return [
    //   cas(color, 1, r)
    // ];

    export function insertBox(box: Box);
    // return [
    //   insert(box)
    // ];

    export function insertOwnedBox(box: OwnedBox);
    // return [
    //   touch(box.owner),
    //   insert(box)
    // ];

    export function deleteBox(box: Ref<Box>);
      // return [
      //   delete(box)
      //  // does:
      //   ...box.data.getAllNodes().map(ref => delete(ref))
      // ];

    export function deleteOwnedBox(box: Ref<OwnedBox>);
      // return [
      //   touch(box.owner),
      //   delete(box)
      // ];
  }
}

const ObjectInstanceMap = {
  0: X.Color,
  1: X.Shape
};

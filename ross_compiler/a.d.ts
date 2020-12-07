interface CreatePatch {
    type: "create";
    id: Hash16;
    data: ObjectRawData;
}
interface DeletePatch {
    type: "delete";
    id: Hash16;
    version: number;
}
interface CASPatch {
    type: "cas";
    id: Hash16;
    base: PrimitiveValue;
    target: PrimitiveValue;
}
type Patch = CreatePatch | DeletePatch | CASPatch;
interface BatchPatch {
    patches: Patch[];
    author?: Hash16;
    action: number;
    time: number;
}
/**
 * X
 */
declare class Snapshot {
    /**
     * The latest version of all the objects in the snapshot.
     */
    readonly objects: Record<Hash16, Ref<any>>;
}
declare class RawReader {
    readonly snapshot: Snapshot;
    readonly data: PrimitiveValue[];
    private cursor;
    constructor(snapshot: Snapshot, data: PrimitiveValue[]);
    next(): PrimitiveValue;
}
/**
 * A 16-byte hash which is stored as a 32-char string in the client.
 */
type Hash16 = string;
/**
 * A pointer to an object that is stored on the server.
 */
interface Ref<T> {
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
type PrimitiveValue = boolean | string | number | Hash16;
type ObjectRawData = [
    number,
    ...PrimitiveValue[]
];
interface StructConstructor<T = any> {
    $: Field[];
    decode(reader: RawReader): T;
    new (): T;
}
type Field =
// Primitive
string | [
    string,
    StructConstructor
] | [
    string
];
declare namespace __ {
    interface CreatePatch {
        type: "create";
        id: Hash16;
        data: ObjectRawData;
    }
    interface DeletePatch {
        type: "delete";
        id: Hash16;
        version: number;
    }
    interface CASPatch {
        type: "cas";
        id: Hash16;
        base: PrimitiveValue;
        target: PrimitiveValue;
    }
    type Patch = CreatePatch | DeletePatch | CASPatch;
    interface BatchPatch {
        patches: Patch[];
        author?: Hash16;
        action: number;
        time: number;
    }
    /**
     * X
     */
    class Snapshot {
        /**
         * The latest version of all the objects in the snapshot.
         */
        readonly objects: Record<Hash16, Ref<any>>;
    }
    class RawReader {
        readonly snapshot: Snapshot;
        readonly data: PrimitiveValue[];
        private cursor;
        constructor(snapshot: Snapshot, data: PrimitiveValue[]);
        next(): PrimitiveValue;
    }
    /**
     * A 16-byte hash which is stored as a 32-char string in the client.
     */
    type Hash16 = string;
    /**
     * A pointer to an object that is stored on the server.
     */
    interface Ref<T> {
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
    type PrimitiveValue = boolean | string | number | Hash16;
    type ObjectRawData = [
        number,
        ...PrimitiveValue[]
    ];
    interface StructConstructor<T = any> {
        $: Field[];
        decode(reader: RawReader): T;
        new (): T;
    }
    type Field =
    // Primitive
    string | [
        string,
        StructConstructor
    ] | [
        string
    ];
    function c(ns: any, id: number, name: string, fields: Field[], members?: string[]): void;
}
export { Hash16, Ref, PrimitiveValue, ObjectRawData, StructConstructor, Field, RawReader, CreatePatch, DeletePatch, CASPatch, Patch, BatchPatch, Snapshot, __ };
export declare namespace root {
    export const _: Record<number, any>;
    export class Point {
        static readonly $: Field[];
        readonly x: number;
        readonly y: number;
        constructor(
            x: number,
            y: number,
        );
    }
    export class Circle {
        static readonly $: Field[];
        readonly center: Point;
        readonly radius: number;
        constructor(
            center: Point,
            radius: number,
        );
    }
    export namespace actions {
        export function insertPoint(point: Point): BatchPatch;
        export function deletePoint(point: Ref<Point>): BatchPatch;
    }
    export namespace colors {
        export class RGB {
            static readonly $: Field[];
            readonly r: number;
            readonly g: number;
            readonly b: number;
            constructor(
                r: number,
                g: number,
                b: number,
            );
        }
        export class CMYK {
            static readonly $: Field[];
            readonly c: number;
            readonly m: number;
            readonly y: number;
            readonly k: number;
            constructor(
                c: number,
                m: number,
                y: number,
                k: number,
            );
        }
        export namespace actions {
        }
    }
}


"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.core = void 0;
var core;
(function (core) {
    class Snapshot {
        constructor() {
            this.objects = Object.create(null);
        }
    }
    core.Snapshot = Snapshot;
    class RawReader {
        constructor(snapshot, data) {
            this.snapshot = snapshot;
            this.data = data;
            this.cursor = 0;
        }
        next() {
            return this.data[this.cursor++];
        }
    }
    core.RawReader = RawReader;
    class Session {
    }
    core.Session = Session;
})(core = exports.core || (exports.core = {}));
function flattenPath(fields) {
    const result = [];
    function write(path, field) {
        if (typeof field === "string" || field[1] === undefined) {
            result.push([...path, field]);
        }
        else {
            const newPath = [...path, field[0]];
            const fields = field[1].$;
            for (let i = 0, n = fields.length; i < n; ++i)
                write(newPath, fields[i]);
        }
    }
    for (let i = 0, n = fields.length; i < n; ++i)
        write([], fields[i]);
    return result;
}
function c(ns, id, name, fields, members = []) {
    let flattenCache;
    const r = function (...args) {
        for (let i = 0, n = fields.length; i < n; ++i) {
            const field = fields[i];
            const key = typeof field === "string" ? field : field[0];
            this[key] = args[i];
        }
        for (let i = 0, n = members.length; i < n; ++i)
            this[members[i]] = [];
    };
    r.prototype.getAllChildren = function () {
        return Array.prototype.concat.apply([], members.map((m) => this[m]));
    };
    r.prototype.getPathFor = function (fieldId) {
        if (!flattenCache)
            flattenCache = flattenPath(fields);
        return flattenCache[fieldId];
    };
    r.$ = fields;
    r.decode = function (reader) {
        const values = [];
        for (let i = 0, n = fields.length; i < n; ++i) {
            const field = fields[i];
            if (typeof field === 'string') {
                values.push(reader.next());
            }
            else if (field[1] === undefined) {
                const id = reader.next();
                if (typeof id !== 'string')
                    throw new TypeError('Expected Hash16.');
                values.push(reader.snapshot.objects[id]);
            }
            else {
                values.push(field[1].decode(reader));
            }
        }
        return new r(...values);
    };
    Object.defineProperty(r, 'name', { value: name });
    ns._[id] = ns[name] = r;
}
exports.root = ($ => {
    $._ = {};
    c($, 0, 'Point', ['x', 'y', ]);
    c($, 1, 'Circle', [['center', $.Point], 'radius', ]);
    const $$ = $.actions = Object.create(null);
    $$.insertPoint = (point) => p(0,
        i(point),
    );
    $$.deletePoint = (point) => p(1,
        d(point),
    );
    return $;
})(Object.create(null));


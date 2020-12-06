"use strict";
exports.__esModule = true;
exports.X = exports.SessionConnection = exports.Snapshot = void 0;
/**
 * Collection of all the objects.
 */
var Snapshot = /** @class */ (function () {
    function Snapshot(objects) {
        this.objects = objects;
    }
    /**
     * Return the object with the given uuid.
     * @param uuid The uuid of the object.
     */
    Snapshot.prototype.getObjectRaw = function (uuid) {
        var info = this.objects[uuid];
        if (info)
            return info.data;
        return null;
    };
    Snapshot.prototype.getObject = function (uuid) { };
    return Snapshot;
}());
exports.Snapshot = Snapshot;
var SessionConnection = /** @class */ (function () {
    function SessionConnection() {
    }
    return SessionConnection;
}());
exports.SessionConnection = SessionConnection;
function generateClass(name, properties, ownedMembers) {
    if (ownedMembers === void 0) { ownedMembers = []; }
    function constructor() {
        var args = [];
        for (var _i = 0; _i < arguments.length; _i++) {
            args[_i] = arguments[_i];
        }
        for (var i = 0, n = properties.length; i < n; ++i) {
            var p = properties[i];
            var key = typeof p === "string" ? p : p[0];
            this[key] = args[i];
        }
        for (var _a = 0, ownedMembers_1 = ownedMembers; _a < ownedMembers_1.length; _a++) {
            var key = ownedMembers_1[_a];
            this[key] = [];
        }
    }
    constructor.$ = properties;
    constructor.name = name;
    return constructor;
}
function insert(obj) {
    return {
        type: "insert"
    };
}
var X;
(function (X) {
    X.Color = generateClass('Color', ['r', 'g', 'b']);
    X.Shape = generateClass('Shape', [['color', X.Color.$], 'size']);
    X.Box = generateClass('Box', ['title'], ['members']);
    X.OwnedBox = generateClass('OwnedBox', ['owner', ['color', X.Color.$]]);
    X.actions = {
        insertColor: function (a) { return [insert(a)]; },
        changeR: function (a, b) { return [cas(a, 1, b)]; }
    };
})(X = exports.X || (exports.X = {}));
var ObjectInstanceMap = {
    0: X.Color,
    1: X.Shape
};

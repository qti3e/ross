const { root, core } = require('./a');
const p = new root.Point(10, 12);
const c = new root.Circle(p, 17);
console.log(c);

const raw = new core.RawReader(null, [10, 12, 17]);
console.log(root._[1].decode(raw));

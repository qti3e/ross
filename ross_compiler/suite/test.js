const test = require("ava");
const { execSync } = require("child_process");

execSync("../target/debug/ross_compiler gen ross/circle.ross dist/circle");
execSync("../target/debug/ross_compiler gen ross/scene.ross dist/scene");

test("object constructor", (t) => {
  const {
    root: { Point2D, Circle },
  } = require("./dist/circle/client");

  const p = new Point2D(10, 12);
  const c = new Circle(p, 20);
  t.is(p.x, 10);
  t.is(p.y, 12);
  t.is(c.center, p);
  t.is(c.radius, 20);
});

test("object raw encoder", (t) => {
  const {
    root: { Point2D, Circle },
  } = require("./dist/circle/client");

  const p = new Point2D(10, 12);
  const c = new Circle(p, 20);
  t.deepEqual(p.encode(), [0, 10, 12]);
  t.deepEqual(c.encode(), [1, 10, 12, 20]);
});

test("getPath", (t) => {
  const {
    root: { Point2D, Circle },
  } = require("./dist/circle/client");

  const p = new Point2D(10, 12);
  const c = new Circle(p, 20);
  t.deepEqual(c.getPathFor(0), ["center", "x"]);
  t.deepEqual(c.getPathFor(1), ["center", "y"]);
  t.deepEqual(c.getPathFor(2), ["radius"]);
});

test("decode", (t) => {
  const {
    root: { Point2D, Circle },
    decode,
  } = require("./dist/circle/client");

  const p = new Point2D(10, 12);
  const c = new Circle(p, 20);
  t.deepEqual(p, decode(null, p.encode()));
  t.deepEqual(c, decode(null, c.encode()));
});

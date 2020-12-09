import ts from "@wessberg/rollup-plugin-ts";

const config = [
  {
    input: "./src/lib.ts",
    output: [{ file: "dist/bundle.js", format: "commonjs" }],
    plugins: [ts({})]
  }
];

export default config;

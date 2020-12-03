const RUST = [
  'abstract',  'else',      'import',    'super',
  'as',        'enum',      'in',        'switch',
  'assert',    'export',    'interface', 'sync',
  'async',     'extends',   'is',        'this',
  'await',     'extension', 'library',   'throw',
  'break',     'external',  'mixin',     'true',
  'case',      'factory',   'new',       'try',
  'catch',     'false',     'null',      'typedef',
  'class',     'final',     'on',        'var',
  'const',     'finally',   'operator',  'void',
  'continue',  'for',       'part',      'while',
  'covariant', 'Function',  'rethrow',   'with',
  'default',   'get',       'return',    'yield',
  'deferred',  'hide',      'set',       'do',
  'if',        'show',      'dynamic',   'implements',
  'static'
];

const JS = [
  'abstract',   'arguments',    'await',     'boolean',
  'break',      'byte',         'casecatch', 'char',
  'class',      'const',        'continue',  'debugger',
  'default',    'delete',       'do',        'double',
  'else',       'enum',         'eval',      'export',
  'extends',    'false',        'final',     'finally',
  'float',      'for',          'function',  'goto',
  'if',         'implements',   'import',    'in',
  'instanceof', 'int',          'interface', 'let',
  'long',       'native',       'new',       'null',
  'package',    'private',      'protected', 'public',
  'return',     'short',        'static',    'super',
  'switch',     'synchronized', 'this',      'throw',
  'throws',     'transient',    'true',      'try',
  'typeof',     'var',          'void',      'volatile',
  'while',      'with',         'yield'
]

const DART = [
  'abstract',  'else',      'import',    'super',
  'as',        'enum',      'in',        'switch',
  'assert',    'export',    'interface', 'sync',
  'async',     'extends',   'is',        'this',
  'await',     'extension', 'library',   'throw',
  'break',     'external',  'mixin',     'true',
  'case',      'factory',   'new',       'try',
  'catch',     'false',     'null',      'typedef',
  'class',     'final',     'on',        'var',
  'const',     'finally',   'operator',  'void',
  'continue',  'for',       'part',      'while',
  'covariant', 'Function',  'rethrow',   'with',
  'default',   'get',       'return',    'yield',
  'deferred',  'hide',      'set',       'do',
  'if',        'show',      'dynamic',   'implements',
  'static'
];

const ROSS = [
  'ref',    'owned',
  'mod',    'struct',
  'in',     'insert',
  'delete', 'enum',
  'with',   'set',
  'graph'
];

const LINE_LENGTH = 110;
const KEYWORDS = Array.from(new Set([...RUST, ...JS, ...DART, ...ROSS])).sort();
const MAX_LEN = Math.max(...KEYWORDS.map(x => x.length));
const PER_LINE = Math.floor((LINE_LENGTH - 1) / (MAX_LEN + 5))

while (KEYWORDS.length) {
  let line = "";
  for (let i = 0; i < PER_LINE && KEYWORDS.length; ++i) {
    const KEYWORD = KEYWORDS.shift();
    if (KEYWORDS.length) {
      const SPACES = MAX_LEN - KEYWORD.length + 1;
      line += ` "${KEYWORD}"${" ".repeat(SPACES)}|`;
    } else {
      line += ` "${KEYWORD}"`;
    }
  }
  console.log(" " + line);
}

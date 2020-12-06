module.exports.root = ($ => {
  c($, 0, 'Point', ['line', 'column', ]);
  c($, 1, 'Range', [['from', $.Point.$], ['to', $.Point.$], ]);
  const $$ = $.actions = Object.create(null);
  $.colors = ($ => {
    c($, 256, 'Color', ['r', 'g', 'b', ]);
    c($, 257, 'Space', ['title', ], ['shapes', ]);
    c($, 258, 'Shape', ['owner', ['color', $.Color.$], 'size', ]);
    const $$ = $.actions = Object.create(null);
    $$.insertShape = (shape) => p(256, [].concat(
      i(shape),
    ));
    $.prototype = null;
    return $;
  })(Object.create($));
  return $;
})(Object.create(null));

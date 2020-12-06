const root = (($, _) => {
  _ = 'Point';$[_] = gc(_, ['line', 'column', ]);
  _ = 'Range';$[_] = gc(_, [['from', $.Point.$], ['to', $.Point.$], ]);
  const $$ = $.actions = Object.create(null);
  $.colors = ($ => {
    _ = 'Color';$[_] = gc(_, ['r', 'g', 'b', ]);
    _ = 'Space';$[_] = gc(_, ['title', ], ['shapes', ]);
    _ = 'Shape';$[_] = gc(_, ['owner', ['color', $.Color.$], 'size', ]);
    const $$ = $.actions = Object.create(null);
    $$.insertShape = (shape) => [].concat(
      insert(shape),
    );
    $.prototype = null;
    return $;
  })(Object.create($));
  return $;
})(Object.create(null));

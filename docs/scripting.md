# Scripting

VectorCraft runs JavaScript files that drive the app through its commands: the same commands the
menus, the [control channel](control-protocol.md) and agents ([MCP](mcp.md)) use. Run one with
**File › Scripts › Other Script…** (Cmd/Ctrl+F12) on the desktop and the web, or from a terminal:

```text
vectorcraft-cli script grid.js [document.svg] [--out art.png]... [--arg key=value]...
                       [--automation-read-root DIR] [--automation-write-root DIR]
```

Scripts written for Adobe Illustrator (`.jsx`, `app.activeDocument`…) don't run: VectorCraft's
scripts use its own commands.

## A first script

```js
// A 5 × 4 grid of squares.
for (let row = 0; row < 4; row++)
  for (let col = 0; col < 5; col++)
    vc.run("shape.rectangle", { x: 20 + col * 40, y: 20 + row * 40, width: 30, height: 30 });
```

A run is one undo step in each document it changes (Edit › Undo Script: grid.js). If the script
throws, or hits a limit, every document goes back to how it was before it ran: the ones it opened
close, the ones it closed come back, and steps it undid return. The fill and stroke defaults, the
clipboard and the Untitled-N count are restored too. View settings (zoom, the perspective grid's
visibility) aren't.

## The `vc` object

| | |
|---|---|
| `vc.run(id, params?)` | Runs a command and returns its result (drawing commands return `{id}`). A failing command throws an `Error` with the command's message, which `try … catch` can catch. `params` must be JSON: functions, symbols, BigInts, cycles, `NaN` and `Infinity` are a `TypeError`. |
| `vc.document()` | The active document as `document.inspect` describes it (layers top first, objects with `id`, `name`, `kind`, `bounds`, `fill`…, artboards, selection, history), or `null` with none open. |
| `vc.commands()` | Every command as `{id, label, params}`; `params` describes what it takes. `vectorcraft-cli commands` prints the same list. |
| `vc.log(...values)` | Adds a line to the run's log, shown after the run (stderr from the CLI). |
| `vc.args` | The `--arg key=value` pairs, as strings; `{}` from the menu. |
| `alert(message)` | Shows the message after the run (stdout from the CLI). It doesn't wait. |

The script's last expression is its value: the CLI prints it as JSON.

```js
// Recolour everything on the top layer, then report how many.
const ids = vc.document().layers[0].children.map((c) => c.id);
vc.run("paint.setFill", { ids, color: "#00aa55" });
ids.length;
```

## Limits

A script can't reach files, the network or the system except through commands (File › Save,
Export…), which behave as they do from the menus; `vectorcraft-cli script` can confine them with
`--automation-read-root` / `--automation-write-root`. A run stops with an error, and changes
nothing, when it:

- goes round loops more than 10,000,000 times in all (every loop counts towards the same total);
- nests calls more than 512 deep;
- runs more than 100,000 commands;
- runs more than 2,000,000,000 JavaScript instructions (about eight seconds of work in the desktop
  app);
- is longer than 1 MiB.

The script can't catch these. A script is also refused before it runs when it nests more than 256
levels of brackets or chains more than about 2,500 operators in one expression (Boa's parser would
run out of stack); so is text that deep handed to `eval`, `new Function` or `JSON.parse`. Values
passed to commands may hold up to 1,000,000 entries in all.

Built-in functions (`Array.from`, `String.prototype.repeat`, `JSON.stringify`, sorting…) do their
work outside these limits, and there is no memory cap: one given an enormous input (a billion items,
or data built a hundred thousand levels deep) can freeze the app, use up its memory or crash it.
Run scripts you trust, as you would any program.

## Language

Modern JavaScript through the [Boa](https://boajs.dev) engine: classes with private fields,
generators, destructuring, spread, `Map`/`Set`, typed arrays and `DataView`, `BigInt`, regular
expressions with named groups, template literals, optional chaining, `JSON`, `Math`, `Date`, and
the legacy web features browsers keep (`substr`, `escape`, `__proto__`…). Promises run to completion
before the script ends: a rejection nothing handles (a throw in an `async` function) fails the run,
and a script whose last expression is a promise has its value. Not available: `import` and modules, `Intl`, `Temporal`, timers
(`setTimeout`), `fetch`, the DOM and `console` (use `vc.log`).

Plain JavaScript libraries that don't need a browser work when pasted in front of a script. The test
corpus runs tinycolor, Alea (seedrandom), simplex-noise and bezier-easing that way
(`crates/script/tests`).

## Errors

The menu shows a dialog with `file.js:LINE:COL: message` and the log; the CLI prints the same on
stderr and exits with code 1. Errors from commands carry the command's message, for example
``unknown command `shape.hexagram` `` or ``invalid parameters for `shape.rectangle`: …``.

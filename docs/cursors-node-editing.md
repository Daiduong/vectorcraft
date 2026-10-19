# Node-editing cursors

> **Last reviewed:** 2026-10-17 · **Last updated:** 2026-10-18 · **Change:** minor (the latest drawings: selection arrows tailless, ReshapeSegment's anchor marks, Curvature's curve; the Pen with its new-path badge and PenDrawing bare). Earlier: minor (the solid and hollow arrows are tailless at rest, a design choice; a confidence number for designers on each researched entry; the toolbar arrows to match, as a to-do). Earlier: minor (stem or tail on the arrows: the drag cursor is a stemless arrowhead, the resting white arrow undecided; two more differences). Earlier: major (first version: every node-editing cursor's meaning, states, look and hotspot from a second pass over the public documentation, checked against the code on `01f85279`) · **Target:** Adobe Illustrator 2026 (30.x)

How the pointer should look and behave while you edit paths: the selection arrows, Direct Selection, Group
Selection, the Pen family, the Anchor Point, Curvature, Add/Delete Anchor Point and Scissors tools. Part of
[ui-parity.md](ui-parity.md) › Cursors.

The cursor list [`crates/tools/src/cursors.toml`](../crates/tools/src/cursors.toml) is what designers draw
from: when each cursor shows, its hotspot, what it must say. It stays short and names no other app. This
page is the reasoning behind it, the evidence for each state, and the differences still left between the
list and the code. When the two disagree, fix whichever is wrong and update both.

## Rules

- **Documented behaviour only.** A cursor state ships when the public documentation describes it. Where
  nothing is documented, VectorCraft keeps the cursor it already has, and this page says so
  (*undocumented: kept*), unless a design choice overrides it (*design choice*). No
  VectorCraft-only cursor states.
- **A cursor never promises a different click.** The cursor at a point must announce what a press there does.
  Where the code's cursor and press disagree, the press is right and the cursor is the bug (see
  [Known differences](#known-differences)).
- **Tailless arrows for manipulating art, a tail for everything else.** On the canvas, the selection
  tools and most tools that manipulate art build their arrow cursors on the tailless arrowheads
  (SelectionArrow, SelectionArrowHollow; the Width tool too). The plain Arrow, with a tail like the
  system pointer, is for the rest; outside the canvas the system pointer shows (design choice, 2026-10-18).
- **One meaning per badge.** A badge (plus, minus, ring, slash, caret, square) means the same thing on
  every cursor that carries it.
- **Hotspots are exact.** The point where the click lands is drawn so a user can aim it at an anchor.
- **Clean-room.** Everything here comes from the text of public pages: help pages, Adobe-published
  manuals, forum posts and tutorials, linked below. No cursor image was looked at, copied or traced, and
  no Adobe software was run. Descriptions are in our own words. Designers draw our cursors from the
  meanings below, never from another app's look.

## Sources

Source tags used below: **doc** (Adobe help pages, manuals or tutorials), **forum** (forum posts,
third-party tutorials), **inferred** (our reading of the evidence).

| Short name | Source |
|---|---|
| CC-PDF | *Adobe Illustrator Help* PDF, last updated 2019-11-04: https://helpx.adobe.com/pdf/illustrator_reference.pdf (page numbers are PDF pages) |
| CS6-PDF | *Using Adobe Illustrator CS6* PDF: http://help.adobe.com/archive/en/illustrator/cs6/illustrator_reference.pdf |
| helpx-segments | https://helpx.adobe.com/illustrator/using/adjust-path-segments.html (Dec 2024 snapshot) |
| helpx-editing | https://helpx.adobe.com/illustrator/using/editing-paths.html (Dec 2024 snapshot) |
| helpx-pen | https://helpx.adobe.com/illustrator/using/drawing-pen-curvature-or-pencil.html (Jul 2024 snapshot) |
| helpx-curv | https://helpx.adobe.com/illustrator/desktop/draw-shapes-and-paths/draw-shapes/draw-shapes-with-the-curvature-tool.html |
| helpx-autoadd | https://helpx.adobe.com/illustrator/desktop/draw-shapes-and-paths/modify-paths/turn-off-automatic-addition-or-deletion-of-anchor-points.html |
| helpx-keys | https://helpx.adobe.com/illustrator/using/default-keyboard-shortcuts.html |
| Learn-pen | Adobe Learn, *Use the Pen tool*, Dec 2025: https://www.adobe.com/learn/illustrator/web/use-pen-tool |
| Learn-curv | Adobe Learn, *Use the Curvature tool*, Feb 2023: https://www.adobe.com/in/learn/illustrator/web/use-curvature-tool |
| SmartNotes | *Illustrator Smart Notes*, pen tool (2022): https://adobeillustratorsmartnotes.com/illustrator/paths/pen-tool.html |
| Lucas | LucasFonts, *Mastering the Pen tool* (2020): https://www.lucasfonts.com/learn/mastering-pen-tool |
| Deke-2013 | https://www.deke.com/content/friday-fundamentals-illustrator-pen-tool-mysteries-decoded |
| Deke-2020 | *Illustrator 2020 One-on-One*, moving and duplicating: https://www.linkedin.com/learning/illustrator-2020-one-on-one-fundamentals/moving-and-duplicating-objects ; Curvature: https://www.linkedin.com/learning/illustrator-2020-one-on-one-fundamentals/drawing-with-the-curvature-tool |
| Burnett | Adobe Community, Feb 2019: https://community.adobe.com/questions-652/has-the-direct-selection-tool-become-harder-to-use-765161 |
| Binder | Adobe Community, Nov 2020: https://community.adobe.com/t5/illustrator-discussions/direct-tool-selecting-whole-path-and-not-just-a-segment-of-the-path/m-p/11591803 |
| Frederiks | Adobe Community, Apr 2024 (Caps Lock): https://community.adobe.com/questions-652/illustrator-pipet-selecteren-mac-809410 |

The help pages Adobe rebuilt in late 2025 (`helpx.adobe.com/illustrator/desktop/…`) describe hardly any
pointers, so most cursor wording now survives in the 2019 PDF and in snapshots of the older pages.

## The cursors

Each entry: what the cursor means, every state that shows it, what the drawing must say, the hotspot,
and the evidence. "Status" says whether the state is documented, and whether the app matches. The
confidence number designers see is in [Confidence](#confidence).

### Arrow (plain pointer, with a tail)
- **Means:** the plain pointer: nothing tool-specific happens here.
- **Shown:** tools with no cursor of their own (Touch Type, Rotate View). Outside the canvas (menus, panels, the toolbar, choosing a tool) the operating
  system's own pointer shows, which designers don't draw.
- **Look:** a solid arrow with a tail, like the system pointer. The arrows with badges for the Gradient
  and Slice Selection tools build on it.
- **Status:** not part of this research (no rating); the selection tools no longer use it.

### SelectionArrow (solid arrowhead, no tail)
- **Means:** select or move the object under the pointer; while dragging, *a move is under way*.
- **Shown:** the Selection tool on the canvas, over empty canvas or an unselected object. Every drag with
  the Selection tool and with Direct Selection (anchors, straight segments, handles, a whole path) until
  the drag snaps. The resting state of the tools that manipulate art (Reshape, Free Transform, Crop
  Image, Puppet Warp, the Perspective tools, Live Paint Selection, Intertwine). The Pen while dragging out handles (the pointer "changes to an arrowhead", helpx-pen).
- **Look:** a solid arrowhead with no tail, at rest and while dragging, a design choice
  (2026-10-18) from public tutorial videos: on the canvas the selection tools' pointers are tailless,
  everything else keeps the tailed Arrow. The text partly disagrees: two sources say the black arrow has
  a tail at rest and loses it over a selected object (shapeshed.com; VectorStyler forum, forum). The
  type-on-a-path bracket and type widget badges build on it.
- **Dragging:** the docs call the drag pointer a filled *arrowhead*, hollow when it snaps, and never an
  arrow (CC-PDF p.98; the Pen and Reflect pages say the pointer "changes to an arrowhead", doc;
  sketchpad.net, forum), which SelectionArrow matches.
- **Hotspot:** the arrowhead's tip.
- **Status:** documented (the drag state), the tailless rest state a design choice. Drawn tailless.

### SelectionArrowHollow (hollow arrowhead, no tail)
- **Means:** Direct Selection: pick anchors and segments. Mid-drag: *the drag has snapped*.
- **Shown:**
  - Direct Selection at rest wherever no other cue applies: empty canvas, fills, handles, straight
    segments, frame edges, and with Shift held.
  - A Selection or Direct Selection drag while it snaps to a point or a guide (the filled arrowhead turns
    hollow, CC-PDF p.98).
  - The Selection tool over an anchor with Snap to Point on.
- **Look:** a hollow arrowhead with no tail (white with a dark outline), its tip where SelectionArrow's is.
  DirectAnchor, DirectAnchorSelected, GroupSelection and CornerRadius are built on it.
- **Stem or no stem:** stemless, a design choice (2026-10-18) from public tutorial videos. The text alone doesn't settle the rest state: one 2017 forum post about the
  redesigned Direct Selection tool misses "a sharp little pointer on a stick", which hints the stem went
  in CC 2017, but it may mean the toolbar icon, and the icon and the pointer are known to differ (forum
  2019). The mid-drag snapped state is a hollow *arrowhead* (doc), so stemless fits it too.
- **Hotspot:** the tip.
- **Evidence:** the Direct Selection pointer is white, matching its tool (CC-PDF p.29; forum 2019:
  the toolbar icon turned solid in CC 2017 but the pointer stayed hollow). Hollow on snap (CC-PDF p.98,
  doc).
- **Status:** documented, the stemless shape a design choice. Drawn stemless, with the family built
  on it. Handles and straight segments:
  no badge documented (*kept*).

### DirectAnchor / DirectAnchorSelected (hollow arrowhead with a square)
- **Means:** a click here takes this anchor.
- **Shown:** Direct Selection over an anchor. **DirectAnchor** when the anchor's path isn't selected,
  **DirectAnchorSelected** when it is (whether or not that anchor itself is).
- **Look:** SelectionArrowHollow with a small square badge: hollow for DirectAnchor, filled for DirectAnchorSelected.
- **Hotspot:** the arrow's tip (not the square).
- **Evidence:** the manual says the pointer shows a hollow square for unselected paths and a filled
  square for selected paths (CC-PDF p.129; CS6-PDF p.30, doc). The cue has existed since CS3 and doesn't
  depend on Smart Guides (Layers Magazine, forum). In 29.4 (2025) users still report it (forum).
- **Status:** documented; matches.

### ReshapeSegment (arrowhead with a curve)
- **Means:** a drag here bends this segment.
- **Shown:**
  - Direct Selection over a curved segment, unless its path is selected whole (a drag then moves the
    path); and while it drags a curved segment. Also over a curved segment of a selected blend's spine.
  - The Anchor Point tool over any segment (straight or curved, it bends both) away from anchors and
    handles, and while it reshapes one.
  - The Pen with Alt held over a segment of a selected path, and while it reshapes one.
- **Look:** a solid arrowhead with a small curve between two anchor marks.
- **Hotspot:** the arrowhead's tip.
- **Evidence:** the Reshape Segment cursor is named in the docs for Pen+Alt and the Anchor Point tool
  (CC-PDF p.397, p.403; helpx-editing, doc). The 2025 help page calls it "the Reshape tool" cursor (doc). Described
  as an arrowhead with a curve between two anchor points (Lynda), and as a black arrow with a small curve
  (Lucas), with black anchor marks (Burnett) (forum). Direct Selection moves a straight segment and only
  bends a curved one (CC-PDF p.396–398, doc), so no bend cue over straight segments (inferred). No curve cue
  when both ends of the segment are selected (Burnett, forum).
- **Status:** documented. Drawn as the tailless arrowhead with a curve between two anchor marks.

### GroupSelection (hollow arrowhead with a plus)
- **Means:** each click selects the next enclosing group.
- **Shown:** the Group Selection tool, at rest and dragging. Direct Selection with Alt held at rest (Alt
  gives Group Selection for the next click).
- **Look:** SelectionArrowHollow with a plus badge.
- **Hotspot:** the tip.
- **Evidence:** white arrow with a plus (Binder; krankykids, forum). Alt switches Direct Selection to Group
  Selection (CC-PDF p.673; helpx-keys, doc). No hover states documented (*kept*).
- **Status:** documented; matches. An Alt-drag copy with Group Selection keeps this cursor (undocumented:
  kept).

### SelectionCopy (two arrowheads)
- **Means:** this drag leaves a copy behind.
- **Shown:** an Alt-drag copy with the Selection tool, and with Direct Selection (an Alt-press on a path
  takes the whole path, and the drag copies it). Pressing Alt during a move switches to it.
- **Look:** two arrowheads, one solid and one hollow, one in front of the other. **When the copy snaps,
  both are hollow** (the same snap rule as SelectionArrow).
- **Hotspot:** the front arrowhead's tip.
- **Evidence:** the cursor changes to show the copy (*Illustrator Smart Notes*, selection page,
  http://www.adobeillustratorsmartnotes.com/illustrator/selection/selection.html, forum). One black and one white
  arrowhead without a snap, both white with a snap (Deke-2020, forum). The Direct Selection cursor plus
  the Selection cursor (Treehouse 2014, forum).
- **Status:** forum-level. The snapped state is missing ([difference 5](#known-differences)).

### CornerRadius (arrow with a rounded corner)
- **Means:** a drag here rounds the corners (Live Corners).
- **Shown:** Selection and Direct Selection over a Live Corners widget, and while dragging one.
- **Evidence:** a curve shows under the arrow over a widget (forum, 2020). While dragging, Up/Down
  changes the corner style and the cursor shows the current style (CC-PDF p.395, doc).
- **Status:** hover matches. *Missing:* a cursor per corner style while dragging (Round, Inverted Round,
  Chamfer), listed in [gaps.md](gaps.md) G1.

### Move (four-way arrow), as the node tools borrow it

The Selection tool shows Move over a selected object, but the sources describe the black arrow losing
its tail there, with a small bounding-box badge since CC 2018 (shapeshed.com; VectorStyler forum;
forum Dec 2017) ([difference 10](#known-differences)).

- **Shown in node editing:** the Curvature tool over a point (a drag moves it); the Reshape tool over a
  selected path.
- **Evidence:** Curvature over a point: "an arrow cursor" (Deke-2020) or a small circle with a dot
  (SmartNotes); the two sources disagree (forum). *Undocumented shape: kept.*

### Pen (nib with an asterisk)
- **Means:** a click here starts a new path.
- **Shown:** the Pen tool when no path is being drawn and no other badge applies, including over a selected
  path with Shift held or Auto Add/Delete off, and with Alt held away from selected paths.
- **Look:** the pen nib with a small asterisk badge. Every Pen-family cursor is this nib with a
  different badge; PenDrawing is the bare nib.
- **Hotspot:** the nib's tip.
- **Evidence:** the asterisk next to the pointer means a new path starts (Learn-pen, doc; Lucas,
  SmartNotes, forum). It goes away once you start drawing and comes back after closing, Esc or deselecting
  (SmartNotes, forum). Forum reports of the pen turning into an x are the precise cursor, not this badge
  (forum 2017–2025).
- **Status:** documented. Drawn.

### PenDrawing (bare nib)
- **Means:** a click here adds the next point of the path being drawn.
- **Shown:** the Pen while drawing, away from points that have a badge of their own (close, convert, join).
- **Look:** the bare nib: the Pen without its asterisk.
- **Evidence:** the pen nib with no extra mark while a path is active (Deke-2013; SmartNotes, forum).
- **Status:** documented. Drawn.

### PenAdd (nib with a plus) / PenDelete (nib with a minus)
- **Means:** a click here adds an anchor / removes this anchor.
- **Shown:**
  - The Pen, when no path is being drawn, over a segment (PenAdd) or an anchor (PenDelete) of a selected
    path, with Auto Add/Delete on and Shift up. Also over a selected blend's spine and its points.
  - The Add and Delete Anchor Point tools everywhere; Alt swaps them.
  - Curvature over a segment of a selected path (PenAdd), and the Mesh tool with Alt over a mesh point (PenDelete).
- **Evidence:** the Pen changes to the Add / Delete Anchor Point tool over a selected path / an anchor
  (helpx-editing, doc). Shift held while hovering turns this off (helpx-autoadd, doc). Alt swaps the two
  tools (CC-PDF p.673, doc). Curvature shows a plus over a segment (Learn-curv, doc).
- **Status:** documented; matches.

### PenClose (nib with a ring)
- **Means:** a click here closes the path.
- **Shown:** the Pen while drawing, over the path's first anchor once it has two or more. The Curvature
  tool over the first point of the path being drawn (three points or more) and, with an open end
  current, near the path's other end.
- **Evidence:** a small circle next to the Pen and the Curvature pointer (helpx-pen; Learn-curv, doc).
- **Status:** documented. Curvature shows it with Alt too, where the click toggles the point
  ([difference 2](#known-differences)).

### PenContinue (nib with a slash)
- **Means:** a click here continues this open path.
- **Shown:** the Pen, no path being drawn, over an end of an open path. Curvature over an open end of a
  selected path.
- **Evidence:** the pointer changes over the endpoint (helpx-segments, doc); a slash next to the nib
  (forum 2017; SmartNotes, forum). SmartNotes says the path must be selected; another 2017 report
  disagrees (forum).
- **Status:** documented; matches.

### PenJoin (nib with a merge badge)
- **Means:** a click here joins the path being drawn to this other path's end.
- **Shown:** the Pen while drawing, over an end of another open path.
- **Look:** a badge showing two path ends meeting at one point.
- **Evidence:** a small merge symbol next to the pointer (helpx-segments; CC-PDF p.155, doc); an endpoint
  with two path stubs (Deke-2013, forum).
- **Status:** documented; matches.

### PenConvert (nib with a caret)
- **Means:** a click on this anchor retracts its outgoing handle (makes the next segment start as a corner).
- **Shown:** the Pen while drawing, over the last anchor, without Alt. The Curvature tool with Alt over
  a point (a click toggles it between corner and smooth).
- **Look:** the nib with a caret badge. It is the Pen's own mark, not the Anchor Point tool's cursor
  (SmartNotes).
- **Evidence:** a convert-point icon appears next to the Pen over its last anchor (helpx-pen; 2025 pages
  on curves followed by straight lines, doc). Curvature with Alt: a little line next to the pointer
  marks the corner the click will make (Learn-curv, doc). VectorCraft uses the caret there (*kept*,
  the mark is unclear).
- **Status:** documented. With Alt over the last anchor the Pen still shows this, though the click
  converts as the Anchor Point tool does ([difference 1](#known-differences)).

### AnchorPoint (caret)
- **Means:** a click converts this anchor between corner and smooth; a drag pulls or splits handles.
- **Shown:** the Anchor Point tool except over a segment (ReshapeSegment there), and while it converts an
  anchor or drags a handle. The Pen with Alt held over an anchor or a handle end of a selected path,
  including the last anchor of the path being drawn.
- **Look:** a caret (an upside-down V), like the tool's own icon, with the hotspot marked at its apex.
- **Hotspot:** the caret's apex.
- **Evidence:** the pointer matches the tool icon (CC-PDF p.29, doc), an upside-down V (Lucas;
  krankykids, forum). Pen+Alt gives the Anchor Point tool (CS6-PDF p.497; helpx-keys, doc). No change over
  an anchor or a handle, and no drag cursor, is documented (*kept*). No hotspot is documented.
- **Status:** documented; matches, except [difference 1](#known-differences).

### Curvature (nib with a curve)
- **Means:** a click here starts a new path or adds the next point.
- **Shown:** the Curvature tool over empty canvas or unselected art, and while drawing away from the
  path's points. Elsewhere it borrows: PenAdd, Move, PenContinue, PenConvert (Alt), PenClose (see above).
- **Look:** the pen nib with a curve badge.
- **Evidence:** at rest the pointer carries an asterisk for a new path (Deke-2020, forum).
  VectorCraft keeps its own curve-badged nib for the tool (*kept*; see Open questions).
- **Status:** drawn with a curve badge; the tool still ignores its drag state ([difference 7](#known-differences)).

### Scissors / ScissorsNotAllowed
- **Means:** a click here splits the path / a click here does nothing.
- **Shown:** the Scissors tool over a segment or an anchor / away from them.
- **Evidence:** Scissors splits at a segment or an anchor; a click elsewhere brings up an alert
  (CC-PDF, doc). No pointer described (*kept*).
- **Status:** [difference 8](#known-differences).

### Crosshair under Use Precise Cursors
- Use Precise Cursors (General preferences) or Caps Lock turns "most tools" into a crosshair centred on
  the hotspot (CC-PDF p.29; CS6-PDF p.55, doc). Named: Pen, Pencil, brushes, Eyedropper, Lasso (doc,
  forum). **Caps Lock inverts the preference**: with it off Caps Lock gives the crosshair, with it on the
  normal cursors (Frederiks, forum).
- No source puts the selection arrows on the crosshair (inferred: they stay). In VectorCraft the Pen family,
  AnchorPoint, Curvature and Scissors become the crosshair; ReshapeSegment, Move and the arrows don't.
- **Status:** the preference works; Caps Lock does nothing yet ([difference 6](#known-differences)).

## By tool

### Direct Selection
| Pointer over / doing | Cursor |
|---|---|
| Empty canvas, a fill, a handle, a straight segment, a frame edge; Shift held | SelectionArrowHollow |
| An anchor: path unselected / path selected | DirectAnchor / DirectAnchorSelected |
| A curved segment (path not selected whole), a curved spine segment | ReshapeSegment |
| A Live Corners widget, or dragging one | CornerRadius |
| Alt held at rest | GroupSelection |
| Dragging anchors, a straight segment, a handle, a whole path | SelectionArrow; SelectionArrowHollow while snapped |
| Dragging a curved segment | ReshapeSegment |
| Alt-dragging (a copy of the whole path) | SelectionCopy (both hollow when snapped: missing) |

### Group Selection
GroupSelection everywhere, dragging included.

### Pen
| Pointer over / doing | No path being drawn | Drawing |
|---|---|---|
| Nothing with a badge | Pen (asterisk) | PenDrawing |
| A segment / an anchor of a selected path, Auto Add/Delete, Shift up | PenAdd / PenDelete | — |
| An end of an open path | PenContinue | PenJoin (another path's end) |
| The first anchor (two or more anchors) | — | PenClose |
| The last anchor, no Alt | — | PenConvert |
| Alt + an anchor or a handle end of a selected path (the last anchor too) | AnchorPoint | AnchorPoint |
| Alt + a segment of a selected path | ReshapeSegment | ReshapeSegment |
| Dragging out handles | SelectionArrow (documented as an arrowhead, helpx-pen) | same |
| Cmd/Ctrl held | the lent selection tool's own cursors (Direct Selection unless another was used last) | same |

Cmd/Ctrl lends Direct Selection in current versions (LinkedIn 2025 training; Deke-2020, forum), and the
lent tool's hover cues show (Lucas, forum). The docs still say "the last used selection tool" (helpx-segments).

### Anchor Point tool
| Pointer over / doing | Cursor |
|---|---|
| An anchor, a handle end, empty canvas | AnchorPoint |
| A segment, away from anchors and handles | ReshapeSegment |
| Converting an anchor, dragging a handle | AnchorPoint |
| Reshaping a segment (Alt copies it, helpx-editing) | ReshapeSegment |

### Curvature
| Pointer over / doing | Cursor |
|---|---|
| Empty canvas, unselected art; drawing away from the points | Curvature |
| A segment of a selected path | PenAdd |
| A point | Move |
| A point with Alt (click toggles corner / smooth) | PenConvert |
| An open end of a selected path | PenContinue |
| The first point of the path being drawn (3+ points, no Alt) | PenClose |
| Dragging a point | Move (no source) |

### Add / Delete Anchor Point, Reshape, Scissors
Add Anchor Point: PenAdd (Alt: PenDelete). Delete Anchor Point: PenDelete (Alt: PenAdd). Reshape: Move
over a selected path or while dragging, SelectionArrow elsewhere (no pointer documented). Scissors: Scissors over a
segment or an anchor, ScissorsNotAllowed elsewhere.

## Known differences

Between this page (and `cursors.toml`) and the code as of 2026-10-18. Each open one is listed in [gaps.md](gaps.md) G1;
resolved ones keep their number so links stay valid.

1. **Pen with Alt over the last anchor shows PenConvert**, but the press converts the anchor as the Anchor
   Point tool does. The cursor checks the path's ends before Alt (`crates/tools/src/pen.rs:410-416`); the
   press checks Alt first (`pen.rs:219`). It should show AnchorPoint.
2. **Curvature with Alt over the first point shows PenClose**, but the press toggles the point between
   corner and smooth (`crates/tools/src/draw2/curvature.rs:621` against `:461`). It should show PenConvert.
3. *Resolved (2026-10-18):* the Pen is drawn with the new-path badge and PenDrawing as the bare nib, so
   starting a path changes the cursor.
4. *Resolved (2026-10-18):* ReshapeSegment is drawn as an arrowhead with a curve between two anchor marks.
5. **No snapped Alt-copy cursor.** SelectionCopy shows whether or not the copy snaps
   (`crates/tools/src/select.rs:563`); a snapped copy should show both arrowheads hollow. It needs a new
   `Cursor` value (CLAUDE.md › Adding a cursor).
6. **Caps Lock doesn't toggle precise cursors.** Only the preference does.
7. **The Curvature tool ignores its drag state** (its drawing now has the curve badge): dragging an
   open end keeps PenContinue rather than Move (`curvature.rs:618-646`).
8. **Scissors: the cursor and the click disagree near anchors.** The press splits at an anchor within
   the point tolerance, but the cursor only tests segments (`crates/tools/src/draw2/anchor.rs:221` against
   `:104`), so a click can cut under ScissorsNotAllowed. The Scissors drawing's hotspot is its crosshair
   centre, not where the blades cross.
9. *Resolved (2026-10-18):* the selection arrows are drawn tailless, solid (SelectionArrow, PathBracket,
   TypeWidget) and hollow (SelectionArrowHollow, DirectAnchor, DirectAnchorSelected, GroupSelection,
   CornerRadius), a design choice. The drag cursors match the documented filled arrowhead that turns
   hollow on a snap (`crates/tools/src/select.rs`, `crates/tools/src/direct.rs` switch SelectionArrow to
   SelectionArrowHollow on a snap).
10. **The Selection tool over a selected object shows the four-way Move cursor.** The sources describe
   the black arrow without its tail there, with a small bounding-box badge since CC 2018: SelectionArrow with a
   badge, a new `Cursor` value.
11. **The toolbar's arrow icons don't match the cursors.** The Selection and Direct Selection toolbar icons
   should be tailless too (public videos show tailless toolbar arrows). A to-do for a
   later change, kept out of this one.

## Confidence

The `confidence` number in `cursors.toml` (0 to 10) tells designers how closely to follow each
description: 9–10 the documentation says it plainly; 7–8 documented, or several independent tutorials
agree; 4–6 one source, or our reading of related evidence; 1–3 nothing public, our own choice; 0 a
placeholder (the full scale is in the file's header). It rates the description as a whole: its
meaning, its look and its hotspot. Cursors outside this page aren't rated yet.

| Cursor | Confidence | Why |
|---|---:|---|
| PenAdd, PenDelete | 9 | plus and minus, documented on help pages for the Pen and the tools |
| PenClose | 9 | the ring, documented for the Pen and Curvature |
| Pen | 8 | the new-path asterisk: an Adobe tutorial plus several independent sources |
| PenDrawing | 8 | the bare nib while drawing: several tutorials |
| SelectionArrow | 6 | tailless by design choice; the drag arrowhead is documented, but two sources give the resting arrow a tail |
| DirectAnchor, DirectAnchorSelected | 7 | the squares are documented; the stemless base is a design choice |
| ReshapeSegment | 7 | named in the docs; its look (arrowhead, curve, two anchor marks, black) from tutorials and forums |
| PenContinue | 7 | the docs say the pointer changes; the slash from several forum posts and a tutorial |
| PenConvert | 7 | documented convert icon; the caret from tutorials |
| AnchorPoint | 7 | the caret is the tool's icon (documented); no hotspot documented |
| SelectionArrowHollow | 6 | hollow on a snap is documented; stemless at rest is a design choice from videos |
| GroupSelection | 6 | the plus from forum posts; the stemless base is a design choice |
| SelectionCopy | 6 | two arrowheads, solid and hollow: one tutorial and one forum post |
| PenJoin | 6 | a merge symbol is documented; its shape from one tutorial |
| CornerRadius | 5 | a curve under the arrow, from forum posts |
| Curvature | 3 | sources describe an asterisk at rest; the curve badge is our choice |
| Scissors, ScissorsNotAllowed | 2 | no pointer described anywhere |

## Open questions

No public source found for these. Until one turns up, the app keeps what it has.

- Direct Selection over a handle end: any badge? (Smart Guides show the word "handle", 2025, forum.)
- Direct Selection over a fill or an unselected object: one 2019 report of a "select the whole object" cue,
  disputed (Burnett).
- The Curvature tool's cursor at rest (an asterisk, per Deke-2020) and over a point ("arrow" or "circle
  with a dot"), its Alt mark ("a little line"), and its cursor while dragging a point.
- The Pen over a lone first point right after placing it: a minus, per SmartNotes only (VectorCraft shows
  PenConvert).
- The Anchor Point tool's hotspot and its cursor while dragging; the Pen with Alt away from paths.
- The Reshape tool's and the Scissors tool's own pointers.
- Whether precise cursors apply to the selection arrows and the Anchor Point tool.

## Revision history

| Date | Change | Summary |
|---|---|---|
| 2026-10-18 | minor | The latest drawings resolve differences 4 and 9 and Curvature's drawing; the Pen drawn with its new-path badge and PenDrawing bare resolves difference 3 |
| 2026-10-18 | minor | The tools that manipulate art (Reshape, Free Transform, Crop Image, Puppet Warp, the Perspective tools, Live Paint Selection, Intertwine) rest on SelectionArrow |
| 2026-10-18 | patch | Rule: tailless arrows for the tools that manipulate art (the Width tool keeps its tailless base), a tail for everything else |
| 2026-10-18 | minor | ArrowHollow renamed SelectionArrowHollow (it belongs to the selection tools); the Pen shows SelectionArrow while dragging out handles, as documented |
| 2026-10-18 | minor | Arrow split in two: Arrow, the tailed plain pointer for other tools, and SelectionArrow, the tailless pointer of the selection tools on the canvas ; the Pen's handle drag showed the nib |
| 2026-10-18 | minor | The solid Arrow is tailless too (design choice), which also makes the drag cursors the documented arrowheads; differences renumbered (9 drawings, 10 Move over a selection, 11 toolbar icons) |
| 2026-10-18 | minor | The hollow arrow family is stemless (design choice); a confidence number (0–10) for designers on each researched cursor-list entry, with the reasons; differences 11 (drawings) and 12 (toolbar arrow icons, a to-do) |
| 2026-10-18 | minor | Stem or tail: the drag cursor is a stemless arrowhead (doc); the Selection tool loses its tail over a selected object (forum); the resting white arrow's stem stays an open question; differences 9 and 10 |
| 2026-10-17 | major | First version: second research pass (prose only) and code audit on `01f85279`; eight known differences, nine open questions |

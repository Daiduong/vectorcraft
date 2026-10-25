# UI and interaction parity

> **Last reviewed:** 2026-10-10 · **Last updated:** 2026-10-24 · **Change:** minor (Recolor Artwork › Assign: double-click a new color for the Color Picker, which keeps a CMYK mix as typed (part of #4028; Lab can't be typed yet); scores and estimates unchanged; previously: docs catch-up: Relative rounding done (#3091), stale Video rulers duplicate removed; scores and estimates unchanged; previously: Select › Start/Stop Global Edit with the Global Edit Options in Properties › Quick Actions, Escape stops it, #3521; previously:; Window › Symbol Libraries and the Symbols panel's Symbol Libraries Menu open symbol library panels; symbols click to add and drag onto the canvas; previously:; Selection tool: in and out ports on every selected area type and type on a path, the in port threading a new or clicked frame before it, #2572; scores unchanged; Smart guides: Distance Guides, Alt/Option over another object or an empty part of an artboard with the Selection tool, and the Invoke Distance Guides preference; scores and estimates unchanged; previously:; Artboard Options: Show Video Safe Areas (`showVideoSafeAreas`) outlines the action-safe and title-safe areas 3.5% and 5% in from each edge (EBU R 95) on the canvas only, and Video Ruler Pixel Aspect Ratio (`videoRulerPixelAspectRatio`, 0.1–10, default 1) sets how wide a video pixel is, per artboard, in one undo step and in the native format; View › Rulers › Show Video Rulers (`view.videoRulers`) counts the active artboard in those pixels from its top-left corner (#2849); scores and estimates unchanged; a double-click on the Line Segment, Arc, Rectangular Grid or Polar Grid tool opens its Tool Options, whose OK sets the next drag's and click's options (#2691, in part); scores and estimates unchanged; Pencil: Shift held at the press draws a straight line at 45° steps from the Constrain Angle (part of #3799); status-bar pointer readout follow-up; scores and hours unchanged; Simplify of part of a path: with anchors or segments of a path direct-selected, Object › Path › Simplify (slider, Auto-Simplify and dialog) refits only each run of selected segments between its end anchors and leaves the rest of the path exactly as it was; the Simplify row is done; scores and estimates unchanged; Fix #3705: in Draw Inside, art drawn or pasted inside the isolated object takes isolation mode into its clip group, so it can be clicked and isn't dimmed; undo or redo that removes the isolated object ends isolation mode; Fix #3705 in part: drawing modes apply to pastes (Draw Inside puts Paste, Paste in Place and Paste on All Artboards inside the object drawn into; Draw Behind puts Paste in Place and Paste on All Artboards behind the selection), and Shift+D goes from Draw Behind back to Draw Normal when Draw Inside can't draw into the selection instead of stopping at Draw Behind; Tangent and straight-segment perpendicular snapping use saved Smart Guides Geometric Guides preferences and Control bar quick access (#2537); no View submenu; G13: gradient Method (Illustrator 30.0): a Method menu in the Gradient panel, Classic or Perceptual (`paint.editGradient`/`paint.setFill` `method`, saved in the native file only when Perceptual, so older files open as Classic); Perceptual mixes a linear or radial gradient's colours in OKLab, on the canvas and in raster exports, and SVG and PDF exports write it as an image at the Document Raster Effects resolution, as the public documentation describes; Classic draws as before; a dithered Perceptual fill takes dither's noise; score and hours unchanged; G13: gradient Dither (Illustrator 30.0): a Dither box in the Gradient panel (`paint.editGradient`/`paint.setFill` `dither`, saved in the native file) draws a linear or radial gradient fill with seeded fine noise that hides banding, on the canvas and in raster exports; SVG and PDF exports write the fill as an image at the Document Raster Effects resolution, as the public documentation describes dither; fills only, as documented; score and hours unchanged; Recolor Artwork Edit chips scroll within the available screen height (#3416); small palettes retain their tab height; scores and estimates unchanged; Blend panel Type and Ramp control step easing with a separate color Shift (#2539); Blend Options retains spacing and orientation; polynomial curves are original approximations; Panels dock into any dock group or the icon column by dragging (UI.6), workspaces keep their layout changes (UI.7), dock groups collapse to their tabs by a double-click (UI.8); scores unchanged; Shape Builder Gap Detection shares Live Paint’s spatial gap finder and temporary merge bridges (#2526); Custom is capped at 72 pt in the existing options dialog; 3D: original Standard, Diffuse, Top Left and Right lighting presets; command access, effective-setting recognition and preview/history; scores and estimates unchanged; one selected clip group's Properties panel has Edit Clipping Path and Edit Contents at its top, G1; Distribute Objects and Distribute Spacing space from a key guide (#3165) with the Distribute Spacing value; Distribute spaces selected ruler guides with the objects, #1597 in part; the selection tools take a clipping path on its outline, over the art it clips; Polygon and Star exact-size placement remembers the chosen count for later drags and dialog openings; Cancel preserves the previous count; Live trace result bounds and selection highlights exclude the retained bitmap except in source-image views; a ruler guide can be the key object Align aligns to, and selected guides align to a key object, #2143; Pencil: Alt key toggles to Smooth Tool, #2001; status-readout sample widths cached by language, document unit and pixels per point; status messages and readouts use stable responsive allocations sized for localized labels and ordinary coordinates; cached canvas dimensions match the renderer limit before cache-key creation, view construction and worker submission; cached canvas reprojection preserves scale during resize, rotation and DPI changes; on-canvas Mirror Repeat axis, #2877; Background Export progress dialog with Cancel, #2564; 3D lighting: one to ten colored lights, smooth colored Inflate, legacy white-light appearance preserved; surface batching and restored seam minimum; scores and estimates unchanged; Blend spine Anchor Point conversions, selection/stacking priority and Shift regression coverage (#2544); scores and hours unchanged; Artboard Options' X and Y with a reference point, #1876; scores unchanged; library panels: one per library, dockable and tabbable, with Persistent, #2603; the Selection tool threads text from the out port, #2572 in part; scores unchanged; Group Selection picks and moves one path of a compound path, #2441 in part; the Move dialog opens on the last move and its last checkboxes; scores unchanged; transparency grid visibility saved in native documents, #1968; Live Corners radius up to the neighbouring cuts, #2332; Image Trace quick presets: wording corrected to the shipped behaviour, the Image Trace button opens the controls; batch exporting, #1935; document window scroll bars, #2322; Anchor Point tool: a click on a handle end retracts that handle only, #2267; scores unchanged; Smart Guides: the dragged end of a straight segment keeps to its extension at any angle, #2120; Warp Options controls verified by focused checks and original-image review; estimates unchanged; existing-guide Escape cancellation validated 2026-10-14; scores and hours unchanged; open-path anchor Delete keeps surviving runs separate; estimates unchanged; Artboard tab-switch follow-up historically validated on `a0671777`; scores and hours unchanged; Artboard clean active-tab close/Delete follow-up validated 2026-10-15; Curvature curve geometry, #1236 in part; Width-point finite-arithmetic follow-up validated; scores and hours unchanged; Image Trace adjustment discovery; later 2026-10-14 merges: macOS save sheet Format menu back to missing, #1855, #866 reopened; anchor X/Y/W/H in the Transform panel, Properties and Control bar, #1824; Snap to Grid over Snap to Pixel, #1816; toolbar labels, columns and the All Tools drawer, #1751; Actions panel selection, #1629 in part; scores unchanged; 2026-10-14 merges: anchor Align and Distribute, #1636; Delete on closed paths and whole objects, #1674; selection inside isolation, #1666; Escape in a Direct Selection drag and while typing, #1743, #1686; Pen clicks under 3 px, #1663; [ and ] on more tools, #1661 in part; fields commit on click away, #1787; command search keys, #1757; a guide's position in the Transform panel is a convenience with no parity claim, #1659; scores unchanged; intersect Smart Guides for drawn points and dragged anchors, #1602 in part; a ruler guide's position in the Transform panel, #1597 in part; Symbol Options before New Symbol, #1560; tool flyout widths, #1616; Pathfinder Trim, Merge and Crop strokes, #1570; scores unchanged; G1/M11: Artboard deletion and history selection normalization; history regressions and full workspace CI pass; scores and hours unchanged; Width Point Edit: Shift-double-click enables adjoining adjustment; plain double-click keeps it off; scores and hours unchanged; per-corner radius fields with a link toggle, split from #1594; Join merges anchors lying on one another, #1533; correction: Average and Join with a corner or smooth choice is missing, not done; guide objects take the guide colour and style and Lock Guides, #1534; Keyboard Shortcuts lists commands outside the menus, #1539; round corners next to curves and the Smooth tool's reach, #1512; Make Compound Path on a group, #1537; dialog slider rails, #1483; template layers stay in Preview in Outline view, #1486; Direct Selection takes the selected object's anchor where anchors coincide, #1508; Tabs panel stop drags, #1458; Split Into Grid dialog, #1462; Transform Objects and Reflect's angle, #1448 and #1460; Pathfinder panel on a group, #1450; value fields: readable selection, #1465 in part, and a focused field follows its value, #1457; scores unchanged; Transform dialogs have a live Preview and step with the arrow keys, #1412; transform tools, dialogs and Move act on direct-selected anchors, #1408; the bounding box hides during a corner drag, #1229; scores unchanged; dialogs focus their first field on every open, #1367, and a first field that starts greyed out once it is enabled, #1395; scores unchanged; Rectangular Grid and Polar Grid click dialogs take the skew, frame, compound path and Fill Grid options, #1377 in part, score unchanged; Live Corners: a Direct Selection click on a widget selects its corner, #1239; custom ruler origin, #1212; Artboard tool clicks fit the topmost editable object in one undo step (#1227); scores unchanged; Pathfinder single-result source names and SVG IDs retained, #1293; DXF control-point SPLINE closure read, #1299 follow-up; Document Setup: Edit Artboards closes the modal after applying valid settings; pointer-click regressions; scores and hours unchanged; Artboards: Escape exits editing; an active drag cancels first; shortcut-routing regression; scores and hours unchanged; Simplify preview/counts and valid rewritten-path selection (#1272); parity remains partial; Texture-effect drag redraws optimized (#1257); idle Windows interaction still unverified; scores unchanged; Curvature point display improved (#1236, in part); scores unchanged; Direct Selection: no-op anchor Align preserves Undo, Redo and saved state (#1180 follow-up); Eyedropper cursor distinguished from precise crosshair (#1269); scores unchanged; Eyedropper can read visible locked and outside-isolation sources (#1251); scores unchanged; Direct Selection: independent segment picks and Delete (#1278); scores unchanged; Adding paint to an unpainted zero-width stroke restores the 1 pt default (#1253); scores unchanged; macOS save sheets offer a Format menu, #866; Attributes fill-rule controls edit the compound when a member is selected, #1197; a marquee needs to touch a path, not just lie inside its fill, #1103; earlier: Live Corners round corners between curved sides, #1144; interactive Mirror Repeat axis; Background Export progress dialog with Cancel, #2564; scores unchanged; 3D lighting: one to ten colored lights, smooth colored Inflate, legacy white-light appearance preserved; surface batching and restored seam minimum; scores and estimates unchanged; Blend spine Anchor Point conversions, selection/stacking priority and Shift regression coverage (#2544); scores and hours unchanged; Artboard Options' X and Y with a reference point, #1876; scores unchanged; library panels: one per library, dockable and tabbable, with Persistent, #2603; the Selection tool threads text from the out port, #2572 in part; scores unchanged; Group Selection picks and moves one path of a compound path, #2441 in part; the Move dialog opens on the last move and its last checkboxes; scores unchanged; transparency grid visibility saved in native documents, #1968; Live Corners radius up to the neighbouring cuts, #2332; Image Trace quick presets: wording corrected to the shipped behaviour, the Image Trace button opens the controls; batch exporting, #1935; document window scroll bars, #2322; Anchor Point tool: a click on a handle end retracts that handle only, #2267; scores unchanged; Smart Guides: the dragged end of a straight segment keeps to its extension at any angle, #2120; Warp Options controls verified by focused checks and original-image review; estimates unchanged; existing-guide Escape cancellation validated 2026-10-14; scores and hours unchanged; open-path anchor Delete keeps surviving runs separate; estimates unchanged; Artboard tab-switch follow-up historically validated on `a0671777`; scores and hours unchanged; Artboard clean active-tab close/Delete follow-up validated 2026-10-15; Curvature curve geometry, #1236 in part; Width-point finite-arithmetic follow-up validated; scores and hours unchanged; Image Trace adjustment discovery; later 2026-10-14 merges: macOS save sheet Format menu back to missing, #1855, #866 reopened; anchor X/Y/W/H in the Transform panel, Properties and Control bar, #1824; Snap to Grid over Snap to Pixel, #1816; toolbar labels, columns and the All Tools drawer, #1751; Actions panel selection, #1629 in part; scores unchanged; 2026-10-14 merges: anchor Align and Distribute, #1636; Delete on closed paths and whole objects, #1674; selection inside isolation, #1666; Escape in a Direct Selection drag and while typing, #1743, #1686; Pen clicks under 3 px, #1663; [ and ] on more tools, #1661 in part; fields commit on click away, #1787; command search keys, #1757; a guide's position in the Transform panel is a convenience with no parity claim, #1659; scores unchanged; intersect Smart Guides for drawn points and dragged anchors, #1602 in part; a ruler guide's position in the Transform panel, #1597 in part; Symbol Options before New Symbol, #1560; tool flyout widths, #1616; Pathfinder Trim, Merge and Crop strokes, #1570; scores unchanged; G1/M11: Artboard deletion and history selection normalization; history regressions and full workspace CI pass; scores and hours unchanged; Width Point Edit: Shift-double-click enables adjoining adjustment; plain double-click keeps it off; scores and hours unchanged; per-corner radius fields with a link toggle, split from #1594; Join merges anchors lying on one another, #1533; correction: Average and Join with a corner or smooth choice is missing, not done; guide objects take the guide colour and style and Lock Guides, #1534; Keyboard Shortcuts lists commands outside the menus, #1539; round corners next to curves and the Smooth tool's reach, #1512; Make Compound Path on a group, #1537; dialog slider rails, #1483; template layers stay in Preview in Outline view, #1486; Direct Selection takes the selected object's anchor where anchors coincide, #1508; Tabs panel stop drags, #1458; Split Into Grid dialog, #1462; Transform Objects and Reflect's angle, #1448 and #1460; Pathfinder panel on a group, #1450; value fields: readable selection, #1465 in part, and a focused field follows its value, #1457; scores unchanged; Transform dialogs have a live Preview and step with the arrow keys, #1412; transform tools, dialogs and Move act on direct-selected anchors, #1408; the bounding box hides during a corner drag, #1229; scores unchanged; dialogs focus their first field on every open, #1367, and a first field that starts greyed out once it is enabled, #1395; scores unchanged; Rectangular Grid and Polar Grid click dialogs take the skew, frame, compound path and Fill Grid options, #1377 in part, score unchanged; Live Corners: a Direct Selection click on a widget selects its corner, #1239; custom ruler origin, #1212; Artboard tool clicks fit the topmost editable object in one undo step (#1227); scores unchanged; Pathfinder single-result source names and SVG IDs retained, #1293; DXF control-point SPLINE closure read, #1299 follow-up; Document Setup: Edit Artboards closes the modal after applying valid settings; pointer-click regressions; scores and hours unchanged; Artboards: Escape exits editing; an active drag cancels first; shortcut-routing regression; scores and hours unchanged; Simplify preview/counts and valid rewritten-path selection (#1272); parity remains partial; Texture-effect drag redraws optimized (#1257); idle Windows interaction still unverified; scores unchanged; Curvature point display improved (#1236, in part); scores unchanged; Direct Selection: no-op anchor Align preserves Undo, Redo and saved state (#1180 follow-up); Eyedropper cursor distinguished from precise crosshair (#1269); scores unchanged; Eyedropper can read visible locked and outside-isolation sources (#1251); scores unchanged; Direct Selection: independent segment picks and Delete (#1278); scores unchanged; Adding paint to an unpainted zero-width stroke restores the 1 pt default (#1253); scores unchanged; macOS save sheets offer a Format menu, #866; Attributes fill-rule controls edit the compound when a member is selected, #1197; a marquee needs to touch a path, not just lie inside its fill, #1103; earlier: Live Corners round corners between curved sides, #1144) · **Target:** Adobe Illustrator 2026 (30.x; bezier drag previews follow the pointer back into the window and to the release, #834; Guides: a path moved by a press on its stroke or beside its edge, and a straight segment Direct Selection drags, snap by the nearest point of the path, landing exactly on a ruler guide (#3701); scores and estimates unchanged; previously:; Artboard Options honors Scale Artwork with Artboard for preset, orientation and typed-size changes; both Fits keep artwork stationary; scores and estimates unchanged; Tangent and straight-segment perpendicular snapping use saved Smart Guides Geometric Guides preferences and Control bar quick access (#2537); no View submenu; G13: gradient Method (Illustrator 30.0): a Method menu in the Gradient panel, Classic or Perceptual (`paint.editGradient`/`paint.setFill` `method`, saved in the native file only when Perceptual, so older files open as Classic); Perceptual mixes a linear or radial gradient's colours in OKLab, on the canvas and in raster exports, and SVG and PDF exports write it as an image at the Document Raster Effects resolution, as the public documentation describes; Classic draws as before; a dithered Perceptual fill takes dither's noise; score and hours unchanged; G13: gradient Dither (Illustrator 30.0): a Dither box in the Gradient panel (`paint.editGradient`/`paint.setFill` `dither`, saved in the native file) draws a linear or radial gradient fill with seeded fine noise that hides banding, on the canvas and in raster exports; SVG and PDF exports write the fill as an image at the Document Raster Effects resolution, as the public documentation describes dither; fills only, as documented; score and hours unchanged; Recolor Artwork Edit chips scroll within the available screen height (#3416); small palettes retain their tab height; scores and estimates unchanged; Blend panel Type and Ramp control step easing with a separate color Shift (#2539); Blend Options retains spacing and orientation; polynomial curves are original approximations; Panels dock into any dock group or the icon column by dragging (UI.6), workspaces keep their layout changes (UI.7), dock groups collapse to their tabs by a double-click (UI.8); scores unchanged; Shape Builder Gap Detection shares Live Paint’s spatial gap finder and temporary merge bridges (#2526); Custom is capped at 72 pt in the existing options dialog; 3D: original Standard, Diffuse, Top Left and Right lighting presets; command access, effective-setting recognition and preview/history; scores and estimates unchanged; one selected clip group's Properties panel has Edit Clipping Path and Edit Contents at its top, G1; Distribute Objects and Distribute Spacing space from a key guide (#3165) with the Distribute Spacing value; Distribute spaces selected ruler guides with the objects, #1597 in part; the selection tools take a clipping path on its outline, over the art it clips; Polygon and Star exact-size placement remembers the chosen count for later drags and dialog openings; Cancel preserves the previous count; Live trace result bounds and selection highlights exclude the retained bitmap except in source-image views; a ruler guide can be the key object Align aligns to, and selected guides align to a key object, #2143; Pencil: Alt key toggles to Smooth Tool, #2001; status-readout sample widths cached by language, document unit and pixels per point; status messages and readouts use stable responsive allocations sized for localized labels and ordinary coordinates; cached canvas dimensions match the renderer limit before cache-key creation, view construction and worker submission; cached canvas reprojection preserves scale during resize, rotation and DPI changes; on-canvas Mirror Repeat axis, #2877; Background Export progress dialog with Cancel, #2564; 3D lighting: one to ten colored lights, smooth colored Inflate, legacy white-light appearance preserved; surface batching and restored seam minimum; scores and estimates unchanged; Blend spine Anchor Point conversions, selection/stacking priority and Shift regression coverage (#2544); scores and hours unchanged; Artboard Options' X and Y with a reference point, #1876; scores unchanged; library panels: one per library, dockable and tabbable, with Persistent, #2603; the Selection tool threads text from the out port, #2572 in part; scores unchanged; Group Selection picks and moves one path of a compound path, #2441 in part; the Move dialog opens on the last move and its last checkboxes; scores unchanged; transparency grid visibility saved in native documents, #1968; Live Corners radius up to the neighbouring cuts, #2332; Image Trace quick presets: wording corrected to the shipped behaviour, the Image Trace button opens the controls; batch exporting, #1935; document window scroll bars, #2322; Anchor Point tool: a click on a handle end retracts that handle only, #2267; scores unchanged; Smart Guides: the dragged end of a straight segment keeps to its extension at any angle, #2120; Warp Options controls verified by focused checks and original-image review; estimates unchanged; existing-guide Escape cancellation validated 2026-10-14; scores and hours unchanged; open-path anchor Delete keeps surviving runs separate; estimates unchanged; Artboard tab-switch follow-up historically validated on `a0671777`; scores and hours unchanged; Artboard clean active-tab close/Delete follow-up validated 2026-10-15; Curvature curve geometry, #1236 in part; Width-point finite-arithmetic follow-up validated; scores and hours unchanged; Image Trace adjustment discovery; later 2026-10-14 merges: macOS save sheet Format menu back to missing, #1855, #866 reopened; anchor X/Y/W/H in the Transform panel, Properties and Control bar, #1824; Snap to Grid over Snap to Pixel, #1816; toolbar labels, columns and the All Tools drawer, #1751; Actions panel selection, #1629 in part; scores unchanged; 2026-10-14 merges: anchor Align and Distribute, #1636; Delete on closed paths and whole objects, #1674; selection inside isolation, #1666; Escape in a Direct Selection drag and while typing, #1743, #1686; Pen clicks under 3 px, #1663; [ and ] on more tools, #1661 in part; fields commit on click away, #1787; command search keys, #1757; a guide's position in the Transform panel is a convenience with no parity claim, #1659; scores unchanged; intersect Smart Guides for drawn points and dragged anchors, #1602 in part; a ruler guide's position in the Transform panel, #1597 in part; Symbol Options before New Symbol, #1560; tool flyout widths, #1616; Pathfinder Trim, Merge and Crop strokes, #1570; scores unchanged; G1/M11: Artboard deletion and history selection normalization; history regressions and full workspace CI pass; scores and hours unchanged; Width Point Edit: Shift-double-click enables adjoining adjustment; plain double-click keeps it off; scores and hours unchanged; per-corner radius fields with a link toggle, split from #1594; Join merges anchors lying on one another, #1533; correction: Average and Join with a corner or smooth choice is missing, not done; guide objects take the guide colour and style and Lock Guides, #1534; Keyboard Shortcuts lists commands outside the menus, #1539; round corners next to curves and the Smooth tool's reach, #1512; Make Compound Path on a group, #1537; dialog slider rails, #1483; template layers stay in Preview in Outline view, #1486; Direct Selection takes the selected object's anchor where anchors coincide, #1508; Tabs panel stop drags, #1458; Split Into Grid dialog, #1462; Transform Objects and Reflect's angle, #1448 and #1460; Pathfinder panel on a group, #1450; value fields: readable selection, #1465 in part, and a focused field follows its value, #1457; scores unchanged; Transform dialogs have a live Preview and step with the arrow keys, #1412; transform tools, dialogs and Move act on direct-selected anchors, #1408; the bounding box hides during a corner drag, #1229; scores unchanged; dialogs focus their first field on every open, #1367, and a first field that starts greyed out once it is enabled, #1395; scores unchanged; Rectangular Grid and Polar Grid click dialogs take the skew, frame, compound path and Fill Grid options, #1377 in part, score unchanged; Live Corners: a Direct Selection click on a widget selects its corner, #1239; custom ruler origin, #1212; Artboard tool clicks fit the topmost editable object in one undo step (#1227); scores unchanged; Pathfinder single-result source names and SVG IDs retained, #1293; DXF control-point SPLINE closure read, #1299 follow-up; Document Setup: Edit Artboards closes the modal after applying valid settings; pointer-click regressions; scores and hours unchanged; Artboards: Escape exits editing; an active drag cancels first; shortcut-routing regression; scores and hours unchanged; Simplify preview/counts and valid rewritten-path selection (#1272); parity remains partial; Texture-effect drag redraws optimized (#1257); idle Windows interaction still unverified; scores unchanged; Curvature point display improved (#1236, in part); scores unchanged; Direct Selection: no-op anchor Align preserves Undo, Redo and saved state (#1180 follow-up); Eyedropper cursor distinguished from precise crosshair (#1269); scores unchanged; Eyedropper can read visible locked and outside-isolation sources (#1251); scores unchanged; Direct Selection: independent segment picks and Delete (#1278); scores unchanged; Adding paint to an unpainted zero-width stroke restores the 1 pt default (#1253); scores unchanged; macOS save sheets offer a Format menu, #866; Attributes fill-rule controls edit the compound when a member is selected, #1197; a marquee needs to touch a path, not just lie inside its fill, #1103; earlier: Live Corners round corners between curved sides, #1144; interactive Mirror Repeat axis; Background Export progress dialog with Cancel, #2564; scores unchanged; 3D lighting: one to ten colored lights, smooth colored Inflate, legacy white-light appearance preserved; surface batching and restored seam minimum; scores and estimates unchanged; Blend spine Anchor Point conversions, selection/stacking priority and Shift regression coverage (#2544); scores and hours unchanged; Artboard Options' X and Y with a reference point, #1876; scores unchanged; library panels: one per library, dockable and tabbable, with Persistent, #2603; the Selection tool threads text from the out port, #2572 in part; scores unchanged; Group Selection picks and moves one path of a compound path, #2441 in part; the Move dialog opens on the last move and its last checkboxes; scores unchanged; transparency grid visibility saved in native documents, #1968; Live Corners radius up to the neighbouring cuts, #2332; Image Trace quick presets: wording corrected to the shipped behaviour, the Image Trace button opens the controls; batch exporting, #1935; document window scroll bars, #2322; Anchor Point tool: a click on a handle end retracts that handle only, #2267; scores unchanged; Smart Guides: the dragged end of a straight segment keeps to its extension at any angle, #2120; Warp Options controls verified by focused checks and original-image review; estimates unchanged; existing-guide Escape cancellation validated 2026-10-14; scores and hours unchanged; open-path anchor Delete keeps surviving runs separate; estimates unchanged; Artboard tab-switch follow-up historically validated on `a0671777`; scores and hours unchanged; Artboard clean active-tab close/Delete follow-up validated 2026-10-15; Curvature curve geometry, #1236 in part; Width-point finite-arithmetic follow-up validated; scores and hours unchanged; Image Trace adjustment discovery; later 2026-10-14 merges: macOS save sheet Format menu back to missing, #1855, #866 reopened; anchor X/Y/W/H in the Transform panel, Properties and Control bar, #1824; Snap to Grid over Snap to Pixel, #1816; toolbar labels, columns and the All Tools drawer, #1751; Actions panel selection, #1629 in part; scores unchanged; 2026-10-14 merges: anchor Align and Distribute, #1636; Delete on closed paths and whole objects, #1674; selection inside isolation, #1666; Escape in a Direct Selection drag and while typing, #1743, #1686; Pen clicks under 3 px, #1663; [ and ] on more tools, #1661 in part; fields commit on click away, #1787; command search keys, #1757; a guide's position in the Transform panel is a convenience with no parity claim, #1659; scores unchanged; intersect Smart Guides for drawn points and dragged anchors, #1602 in part; a ruler guide's position in the Transform panel, #1597 in part; Symbol Options before New Symbol, #1560; tool flyout widths, #1616; Pathfinder Trim, Merge and Crop strokes, #1570; scores unchanged; G1/M11: Artboard deletion and history selection normalization; history regressions and full workspace CI pass; scores and hours unchanged; Width Point Edit: Shift-double-click enables adjoining adjustment; plain double-click keeps it off; scores and hours unchanged; per-corner radius fields with a link toggle, split from #1594; Join merges anchors lying on one another, #1533; correction: Average and Join with a corner or smooth choice is missing, not done; guide objects take the guide colour and style and Lock Guides, #1534; Keyboard Shortcuts lists commands outside the menus, #1539; round corners next to curves and the Smooth tool's reach, #1512; Make Compound Path on a group, #1537; dialog slider rails, #1483; template layers stay in Preview in Outline view, #1486; Direct Selection takes the selected object's anchor where anchors coincide, #1508; Tabs panel stop drags, #1458; Split Into Grid dialog, #1462; Transform Objects and Reflect's angle, #1448 and #1460; Pathfinder panel on a group, #1450; value fields: readable selection, #1465 in part, and a focused field follows its value, #1457; scores unchanged; Transform dialogs have a live Preview and step with the arrow keys, #1412; transform tools, dialogs and Move act on direct-selected anchors, #1408; the bounding box hides during a corner drag, #1229; scores unchanged; dialogs focus their first field on every open, #1367, and a first field that starts greyed out once it is enabled, #1395; scores unchanged; Rectangular Grid and Polar Grid click dialogs take the skew, frame, compound path and Fill Grid options, #1377 in part, score unchanged; Live Corners: a Direct Selection click on a widget selects its corner, #1239; custom ruler origin, #1212; Artboard tool clicks fit the topmost editable object in one undo step (#1227); scores unchanged; Pathfinder single-result source names and SVG IDs retained, #1293; DXF control-point SPLINE closure read, #1299 follow-up; Document Setup: Edit Artboards closes the modal after applying valid settings; pointer-click regressions; scores and hours unchanged; Artboards: Escape exits editing; an active drag cancels first; shortcut-routing regression; scores and hours unchanged; Simplify preview/counts and valid rewritten-path selection (#1272); parity remains partial; Texture-effect drag redraws optimized (#1257); idle Windows interaction still unverified; scores unchanged; Curvature point display improved (#1236, in part); scores unchanged; Direct Selection: no-op anchor Align preserves Undo, Redo and saved state (#1180 follow-up); Eyedropper cursor distinguished from precise crosshair (#1269); scores unchanged; Eyedropper can read visible locked and outside-isolation sources (#1251); scores unchanged; Direct Selection: independent segment picks and Delete (#1278); scores unchanged; Adding paint to an unpainted zero-width stroke restores the 1 pt default (#1253); scores unchanged; macOS save sheets offer a Format menu, #866; Attributes fill-rule controls edit the compound when a member is selected, #1197; a marquee needs to touch a path, not just lie inside its fill, #1103; earlier: Live Corners round corners between curved sides, #1144) · **Target:** Adobe Illustrator 2026 (30.x; Artboard Options: Show Center Mark and Show Cross Hairs (`artboard.setProps`/`artboard.new` `showCenterMark`, `showCrossHairs`) draw a center cross and lines from the middle of each side through the center on the canvas only, in one undo step; video safe areas, video rulers and the pixel aspect ratio remain missing; Tangent and straight-segment perpendicular snapping use saved Smart Guides Geometric Guides preferences and Control bar quick access (#2537); no View submenu; G13: gradient Method (Illustrator 30.0): a Method menu in the Gradient panel, Classic or Perceptual (`paint.editGradient`/`paint.setFill` `method`, saved in the native file only when Perceptual, so older files open as Classic); Perceptual mixes a linear or radial gradient's colours in OKLab, on the canvas and in raster exports, and SVG and PDF exports write it as an image at the Document Raster Effects resolution, as the public documentation describes; Classic draws as before; a dithered Perceptual fill takes dither's noise; score and hours unchanged; G13: gradient Dither (Illustrator 30.0): a Dither box in the Gradient panel (`paint.editGradient`/`paint.setFill` `dither`, saved in the native file) draws a linear or radial gradient fill with seeded fine noise that hides banding, on the canvas and in raster exports; SVG and PDF exports write the fill as an image at the Document Raster Effects resolution, as the public documentation describes dither; fills only, as documented; score and hours unchanged; Recolor Artwork Edit chips scroll within the available screen height (#3416); small palettes retain their tab height; scores and estimates unchanged; Blend panel Type and Ramp control step easing with a separate color Shift (#2539); Blend Options retains spacing and orientation; polynomial curves are original approximations; Panels dock into any dock group or the icon column by dragging (UI.6), workspaces keep their layout changes (UI.7), dock groups collapse to their tabs by a double-click (UI.8); scores unchanged; Shape Builder Gap Detection shares Live Paint’s spatial gap finder and temporary merge bridges (#2526); Custom is capped at 72 pt in the existing options dialog; 3D: original Standard, Diffuse, Top Left and Right lighting presets; command access, effective-setting recognition and preview/history; scores and estimates unchanged; one selected clip group's Properties panel has Edit Clipping Path and Edit Contents at its top, G1; Distribute Objects and Distribute Spacing space from a key guide (#3165) with the Distribute Spacing value; Distribute spaces selected ruler guides with the objects, #1597 in part; the selection tools take a clipping path on its outline, over the art it clips; Polygon and Star exact-size placement remembers the chosen count for later drags and dialog openings; Cancel preserves the previous count; Live trace result bounds and selection highlights exclude the retained bitmap except in source-image views; a ruler guide can be the key object Align aligns to, and selected guides align to a key object, #2143; Pencil: Alt key toggles to Smooth Tool, #2001; status-readout sample widths cached by language, document unit and pixels per point; status messages and readouts use stable responsive allocations sized for localized labels and ordinary coordinates; cached canvas dimensions match the renderer limit before cache-key creation, view construction and worker submission; cached canvas reprojection preserves scale during resize, rotation and DPI changes; on-canvas Mirror Repeat axis, #2877; Background Export progress dialog with Cancel, #2564; 3D lighting: one to ten colored lights, smooth colored Inflate, legacy white-light appearance preserved; surface batching and restored seam minimum; scores and estimates unchanged; Blend spine Anchor Point conversions, selection/stacking priority and Shift regression coverage (#2544); scores and hours unchanged; Artboard Options' X and Y with a reference point, #1876; scores unchanged; library panels: one per library, dockable and tabbable, with Persistent, #2603; the Selection tool threads text from the out port, #2572 in part; scores unchanged; Group Selection picks and moves one path of a compound path, #2441 in part; the Move dialog opens on the last move and its last checkboxes; scores unchanged; transparency grid visibility saved in native documents, #1968; Live Corners radius up to the neighbouring cuts, #2332; Image Trace quick presets: wording corrected to the shipped behaviour, the Image Trace button opens the controls; batch exporting, #1935; document window scroll bars, #2322; Anchor Point tool: a click on a handle end retracts that handle only, #2267; scores unchanged; Smart Guides: the dragged end of a straight segment keeps to its extension at any angle, #2120; Warp Options controls verified by focused checks and original-image review; estimates unchanged; existing-guide Escape cancellation validated 2026-10-14; scores and hours unchanged; open-path anchor Delete keeps surviving runs separate; estimates unchanged; Artboard tab-switch follow-up historically validated on `a0671777`; scores and hours unchanged; Artboard clean active-tab close/Delete follow-up validated 2026-10-15; Curvature curve geometry, #1236 in part; Width-point finite-arithmetic follow-up validated; scores and hours unchanged; Image Trace adjustment discovery; later 2026-10-14 merges: macOS save sheet Format menu back to missing, #1855, #866 reopened; anchor X/Y/W/H in the Transform panel, Properties and Control bar, #1824; Snap to Grid over Snap to Pixel, #1816; toolbar labels, columns and the All Tools drawer, #1751; Actions panel selection, #1629 in part; scores unchanged; 2026-10-14 merges: anchor Align and Distribute, #1636; Delete on closed paths and whole objects, #1674; selection inside isolation, #1666; Escape in a Direct Selection drag and while typing, #1743, #1686; Pen clicks under 3 px, #1663; [ and ] on more tools, #1661 in part; fields commit on click away, #1787; command search keys, #1757; a guide's position in the Transform panel is a convenience with no parity claim, #1659; scores unchanged; intersect Smart Guides for drawn points and dragged anchors, #1602 in part; a ruler guide's position in the Transform panel, #1597 in part; Symbol Options before New Symbol, #1560; tool flyout widths, #1616; Pathfinder Trim, Merge and Crop strokes, #1570; scores unchanged; G1/M11: Artboard deletion and history selection normalization; history regressions and full workspace CI pass; scores and hours unchanged; Width Point Edit: Shift-double-click enables adjoining adjustment; plain double-click keeps it off; scores and hours unchanged; per-corner radius fields with a link toggle, split from #1594; Join merges anchors lying on one another, #1533; correction: Average and Join with a corner or smooth choice is missing, not done; guide objects take the guide colour and style and Lock Guides, #1534; Keyboard Shortcuts lists commands outside the menus, #1539; round corners next to curves and the Smooth tool's reach, #1512; Make Compound Path on a group, #1537; dialog slider rails, #1483; template layers stay in Preview in Outline view, #1486; Direct Selection takes the selected object's anchor where anchors coincide, #1508; Tabs panel stop drags, #1458; Split Into Grid dialog, #1462; Transform Objects and Reflect's angle, #1448 and #1460; Pathfinder panel on a group, #1450; value fields: readable selection, #1465 in part, and a focused field follows its value, #1457; scores unchanged; Transform dialogs have a live Preview and step with the arrow keys, #1412; transform tools, dialogs and Move act on direct-selected anchors, #1408; the bounding box hides during a corner drag, #1229; scores unchanged; dialogs focus their first field on every open, #1367, and a first field that starts greyed out once it is enabled, #1395; scores unchanged; Rectangular Grid and Polar Grid click dialogs take the skew, frame, compound path and Fill Grid options, #1377 in part, score unchanged; Live Corners: a Direct Selection click on a widget selects its corner, #1239; custom ruler origin, #1212; Artboard tool clicks fit the topmost editable object in one undo step (#1227); scores unchanged; Pathfinder single-result source names and SVG IDs retained, #1293; DXF control-point SPLINE closure read, #1299 follow-up; Document Setup: Edit Artboards closes the modal after applying valid settings; pointer-click regressions; scores and hours unchanged; Artboards: Escape exits editing; an active drag cancels first; shortcut-routing regression; scores and hours unchanged; Simplify preview/counts and valid rewritten-path selection (#1272); parity remains partial; Texture-effect drag redraws optimized (#1257); idle Windows interaction still unverified; scores unchanged; Curvature point display improved (#1236, in part); scores unchanged; Direct Selection: no-op anchor Align preserves Undo, Redo and saved state (#1180 follow-up); Eyedropper cursor distinguished from precise crosshair (#1269); scores unchanged; Eyedropper can read visible locked and outside-isolation sources (#1251); scores unchanged; Direct Selection: independent segment picks and Delete (#1278); scores unchanged; Adding paint to an unpainted zero-width stroke restores the 1 pt default (#1253); scores unchanged; macOS save sheets offer a Format menu, #866; Attributes fill-rule controls edit the compound when a member is selected, #1197; a marquee needs to touch a path, not just lie inside its fill, #1103; earlier: Live Corners round corners between curved sides, #1144; interactive Mirror Repeat axis; Background Export progress dialog with Cancel, #2564; scores unchanged; 3D lighting: one to ten colored lights, smooth colored Inflate, legacy white-light appearance preserved; surface batching and restored seam minimum; scores and estimates unchanged; Blend spine Anchor Point conversions, selection/stacking priority and Shift regression coverage (#2544); scores and hours unchanged; Artboard Options' X and Y with a reference point, #1876; scores unchanged; library panels: one per library, dockable and tabbable, with Persistent, #2603; the Selection tool threads text from the out port, #2572 in part; scores unchanged; Group Selection picks and moves one path of a compound path, #2441 in part; the Move dialog opens on the last move and its last checkboxes; scores unchanged; transparency grid visibility saved in native documents, #1968; Live Corners radius up to the neighbouring cuts, #2332; Image Trace quick presets: wording corrected to the shipped behaviour, the Image Trace button opens the controls; batch exporting, #1935; document window scroll bars, #2322; Anchor Point tool: a click on a handle end retracts that handle only, #2267; scores unchanged; Smart Guides: the dragged end of a straight segment keeps to its extension at any angle, #2120; Warp Options controls verified by focused checks and original-image review; estimates unchanged; existing-guide Escape cancellation validated 2026-10-14; scores and hours unchanged; open-path anchor Delete keeps surviving runs separate; estimates unchanged; Artboard tab-switch follow-up historically validated on `a0671777`; scores and hours unchanged; Artboard clean active-tab close/Delete follow-up validated 2026-10-15; Curvature curve geometry, #1236 in part; Width-point finite-arithmetic follow-up validated; scores and hours unchanged; Image Trace adjustment discovery; later 2026-10-14 merges: macOS save sheet Format menu back to missing, #1855, #866 reopened; anchor X/Y/W/H in the Transform panel, Properties and Control bar, #1824; Snap to Grid over Snap to Pixel, #1816; toolbar labels, columns and the All Tools drawer, #1751; Actions panel selection, #1629 in part; scores unchanged; 2026-10-14 merges: anchor Align and Distribute, #1636; Delete on closed paths and whole objects, #1674; selection inside isolation, #1666; Escape in a Direct Selection drag and while typing, #1743, #1686; Pen clicks under 3 px, #1663; [ and ] on more tools, #1661 in part; fields commit on click away, #1787; command search keys, #1757; a guide's position in the Transform panel is a convenience with no parity claim, #1659; scores unchanged; intersect Smart Guides for drawn points and dragged anchors, #1602 in part; a ruler guide's position in the Transform panel, #1597 in part; Symbol Options before New Symbol, #1560; tool flyout widths, #1616; Pathfinder Trim, Merge and Crop strokes, #1570; scores unchanged; G1/M11: Artboard deletion and history selection normalization; history regressions and full workspace CI pass; scores and hours unchanged; Width Point Edit: Shift-double-click enables adjoining adjustment; plain double-click keeps it off; scores and hours unchanged; per-corner radius fields with a link toggle, split from #1594; Join merges anchors lying on one another, #1533; correction: Average and Join with a corner or smooth choice is missing, not done; guide objects take the guide colour and style and Lock Guides, #1534; Keyboard Shortcuts lists commands outside the menus, #1539; round corners next to curves and the Smooth tool's reach, #1512; Make Compound Path on a group, #1537; dialog slider rails, #1483; template layers stay in Preview in Outline view, #1486; Direct Selection takes the selected object's anchor where anchors coincide, #1508; Tabs panel stop drags, #1458; Split Into Grid dialog, #1462; Transform Objects and Reflect's angle, #1448 and #1460; Pathfinder panel on a group, #1450; value fields: readable selection, #1465 in part, and a focused field follows its value, #1457; scores unchanged; Transform dialogs have a live Preview and step with the arrow keys, #1412; transform tools, dialogs and Move act on direct-selected anchors, #1408; the bounding box hides during a corner drag, #1229; scores unchanged; dialogs focus their first field on every open, #1367, and a first field that starts greyed out once it is enabled, #1395; scores unchanged; Rectangular Grid and Polar Grid click dialogs take the skew, frame, compound path and Fill Grid options, #1377 in part, score unchanged; Live Corners: a Direct Selection click on a widget selects its corner, #1239; custom ruler origin, #1212; Artboard tool clicks fit the topmost editable object in one undo step (#1227); scores unchanged; Pathfinder single-result source names and SVG IDs retained, #1293; DXF control-point SPLINE closure read, #1299 follow-up; Document Setup: Edit Artboards closes the modal after applying valid settings; pointer-click regressions; scores and hours unchanged; Artboards: Escape exits editing; an active drag cancels first; shortcut-routing regression; scores and hours unchanged; Simplify preview/counts and valid rewritten-path selection (#1272); parity remains partial; Texture-effect drag redraws optimized (#1257); idle Windows interaction still unverified; scores unchanged; Curvature point display improved (#1236, in part); scores unchanged; Direct Selection: no-op anchor Align preserves Undo, Redo and saved state (#1180 follow-up); Eyedropper cursor distinguished from precise crosshair (#1269); scores unchanged; Eyedropper can read visible locked and outside-isolation sources (#1251); scores unchanged; Direct Selection: independent segment picks and Delete (#1278); scores unchanged; Adding paint to an unpainted zero-width stroke restores the 1 pt default (#1253); scores unchanged; macOS save sheets offer a Format menu, #866; Attributes fill-rule controls edit the compound when a member is selected, #1197; a marquee needs to touch a path, not just lie inside its fill, #1103; earlier: Live Corners round corners between curved sides, #1144) · **Target:** Adobe Illustrator 2026 (30.x; bezier drag previews follow the pointer back into the window and to the release, #834; Guides: a path moved by a press on its stroke or beside its edge, and a straight segment Direct Selection drags, snap by the nearest point of the path, landing exactly on a ruler guide (#3701); scores and estimates unchanged; previously:; Artboard Options honors Scale Artwork with Artboard for preset, orientation and typed-size changes; both Fits keep artwork stationary; scores and estimates unchanged; Tangent and straight-segment perpendicular snapping use saved Smart Guides Geometric Guides preferences and Control bar quick access (#2537); no View submenu; G13: gradient Method (Illustrator 30.0): a Method menu in the Gradient panel, Classic or Perceptual (`paint.editGradient`/`paint.setFill` `method`, saved in the native file only when Perceptual, so older files open as Classic); Perceptual mixes a linear or radial gradient's colours in OKLab, on the canvas and in raster exports, and SVG and PDF exports write it as an image at the Document Raster Effects resolution, as the public documentation describes; Classic draws as before; a dithered Perceptual fill takes dither's noise; score and hours unchanged; G13: gradient Dither (Illustrator 30.0): a Dither box in the Gradient panel (`paint.editGradient`/`paint.setFill` `dither`, saved in the native file) draws a linear or radial gradient fill with seeded fine noise that hides banding, on the canvas and in raster exports; SVG and PDF exports write the fill as an image at the Document Raster Effects resolution, as the public documentation describes dither; fills only, as documented; score and hours unchanged; Recolor Artwork Edit chips scroll within the available screen height (#3416); small palettes retain their tab height; scores and estimates unchanged; Blend panel Type and Ramp control step easing with a separate color Shift (#2539); Blend Options retains spacing and orientation; polynomial curves are original approximations; Panels dock into any dock group or the icon column by dragging (UI.6), workspaces keep their layout changes (UI.7), dock groups collapse to their tabs by a double-click (UI.8); scores unchanged; Shape Builder Gap Detection shares Live Paint’s spatial gap finder and temporary merge bridges (#2526); Custom is capped at 72 pt in the existing options dialog; 3D: original Standard, Diffuse, Top Left and Right lighting presets; command access, effective-setting recognition and preview/history; scores and estimates unchanged; one selected clip group's Properties panel has Edit Clipping Path and Edit Contents at its top, G1; Distribute Objects and Distribute Spacing space from a key guide (#3165) with the Distribute Spacing value; Distribute spaces selected ruler guides with the objects, #1597 in part; the selection tools take a clipping path on its outline, over the art it clips; Polygon and Star exact-size placement remembers the chosen count for later drags and dialog openings; Cancel preserves the previous count; Live trace result bounds and selection highlights exclude the retained bitmap except in source-image views; a ruler guide can be the key object Align aligns to, and selected guides align to a key object, #2143; Pencil: Alt key toggles to Smooth Tool, #2001; status-readout sample widths cached by language, document unit and pixels per point; status messages and readouts use stable responsive allocations sized for localized labels and ordinary coordinates; cached canvas dimensions match the renderer limit before cache-key creation, view construction and worker submission; cached canvas reprojection preserves scale during resize, rotation and DPI changes; on-canvas Mirror Repeat axis, #2877; Background Export progress dialog with Cancel, #2564; 3D lighting: one to ten colored lights, smooth colored Inflate, legacy white-light appearance preserved; surface batching and restored seam minimum; scores and estimates unchanged; Blend spine Anchor Point conversions, selection/stacking priority and Shift regression coverage (#2544); scores and hours unchanged; Artboard Options' X and Y with a reference point, #1876; scores unchanged; library panels: one per library, dockable and tabbable, with Persistent, #2603; the Selection tool threads text from the out port, #2572 in part; scores unchanged; Group Selection picks and moves one path of a compound path, #2441 in part; the Move dialog opens on the last move and its last checkboxes; scores unchanged; transparency grid visibility saved in native documents, #1968; Live Corners radius up to the neighbouring cuts, #2332; Image Trace quick presets: wording corrected to the shipped behaviour, the Image Trace button opens the controls; batch exporting, #1935; document window scroll bars, #2322; Anchor Point tool: a click on a handle end retracts that handle only, #2267; scores unchanged; Smart Guides: the dragged end of a straight segment keeps to its extension at any angle, #2120; Warp Options controls verified by focused checks and original-image review; estimates unchanged; existing-guide Escape cancellation validated 2026-10-14; scores and hours unchanged; open-path anchor Delete keeps surviving runs separate; estimates unchanged; Artboard tab-switch follow-up historically validated on `a0671777`; scores and hours unchanged; Artboard clean active-tab close/Delete follow-up validated 2026-10-15; Curvature curve geometry, #1236 in part; Width-point finite-arithmetic follow-up validated; scores and hours unchanged; Image Trace adjustment discovery; later 2026-10-14 merges: macOS save sheet Format menu back to missing, #1855, #866 reopened; anchor X/Y/W/H in the Transform panel, Properties and Control bar, #1824; Snap to Grid over Snap to Pixel, #1816; toolbar labels, columns and the All Tools drawer, #1751; Actions panel selection, #1629 in part; scores unchanged; 2026-10-14 merges: anchor Align and Distribute, #1636; Delete on closed paths and whole objects, #1674; selection inside isolation, #1666; Escape in a Direct Selection drag and while typing, #1743, #1686; Pen clicks under 3 px, #1663; [ and ] on more tools, #1661 in part; fields commit on click away, #1787; command search keys, #1757; a guide's position in the Transform panel is a convenience with no parity claim, #1659; scores unchanged; intersect Smart Guides for drawn points and dragged anchors, #1602 in part; a ruler guide's position in the Transform panel, #1597 in part; Symbol Options before New Symbol, #1560; tool flyout widths, #1616; Pathfinder Trim, Merge and Crop strokes, #1570; scores unchanged; G1/M11: Artboard deletion and history selection normalization; history regressions and full workspace CI pass; scores and hours unchanged; Width Point Edit: Shift-double-click enables adjoining adjustment; plain double-click keeps it off; scores and hours unchanged; per-corner radius fields with a link toggle, split from #1594; Join merges anchors lying on one another, #1533; correction: Average and Join with a corner or smooth choice is missing, not done; guide objects take the guide colour and style and Lock Guides, #1534; Keyboard Shortcuts lists commands outside the menus, #1539; round corners next to curves and the Smooth tool's reach, #1512; Make Compound Path on a group, #1537; dialog slider rails, #1483; template layers stay in Preview in Outline view, #1486; Direct Selection takes the selected object's anchor where anchors coincide, #1508; Tabs panel stop drags, #1458; Split Into Grid dialog, #1462; Transform Objects and Reflect's angle, #1448 and #1460; Pathfinder panel on a group, #1450; value fields: readable selection, #1465 in part, and a focused field follows its value, #1457; scores unchanged; Transform dialogs have a live Preview and step with the arrow keys, #1412; transform tools, dialogs and Move act on direct-selected anchors, #1408; the bounding box hides during a corner drag, #1229; scores unchanged; dialogs focus their first field on every open, #1367, and a first field that starts greyed out once it is enabled, #1395; scores unchanged; Rectangular Grid and Polar Grid click dialogs take the skew, frame, compound path and Fill Grid options, #1377 in part, score unchanged; Live Corners: a Direct Selection click on a widget selects its corner, #1239; custom ruler origin, #1212; Artboard tool clicks fit the topmost editable object in one undo step (#1227); scores unchanged; Pathfinder single-result source names and SVG IDs retained, #1293; DXF control-point SPLINE closure read, #1299 follow-up; Document Setup: Edit Artboards closes the modal after applying valid settings; pointer-click regressions; scores and hours unchanged; Artboards: Escape exits editing; an active drag cancels first; shortcut-routing regression; scores and hours unchanged; Simplify preview/counts and valid rewritten-path selection (#1272); parity remains partial; Texture-effect drag redraws optimized (#1257); idle Windows interaction still unverified; scores unchanged; Curvature point display improved (#1236, in part); scores unchanged; Direct Selection: no-op anchor Align preserves Undo, Redo and saved state (#1180 follow-up); Eyedropper cursor distinguished from precise crosshair (#1269); scores unchanged; Eyedropper can read visible locked and outside-isolation sources (#1251); scores unchanged; Direct Selection: independent segment picks and Delete (#1278); scores unchanged; Adding paint to an unpainted zero-width stroke restores the 1 pt default (#1253); scores unchanged; macOS save sheets offer a Format menu, #866; Attributes fill-rule controls edit the compound when a member is selected, #1197; a marquee needs to touch a path, not just lie inside its fill, #1103; earlier: Live Corners round corners between curved sides, #1144; interactive Mirror Repeat axis; Background Export progress dialog with Cancel, #2564; scores unchanged; 3D lighting: one to ten colored lights, smooth colored Inflate, legacy white-light appearance preserved; surface batching and restored seam minimum; scores and estimates unchanged; Blend spine Anchor Point conversions, selection/stacking priority and Shift regression coverage (#2544); scores and hours unchanged; Artboard Options' X and Y with a reference point, #1876; scores unchanged; library panels: one per library, dockable and tabbable, with Persistent, #2603; the Selection tool threads text from the out port, #2572 in part; scores unchanged; Group Selection picks and moves one path of a compound path, #2441 in part; the Move dialog opens on the last move and its last checkboxes; scores unchanged; transparency grid visibility saved in native documents, #1968; Live Corners radius up to the neighbouring cuts, #2332; Image Trace quick presets: wording corrected to the shipped behaviour, the Image Trace button opens the controls; batch exporting, #1935; document window scroll bars, #2322; Anchor Point tool: a click on a handle end retracts that handle only, #2267; scores unchanged; Smart Guides: the dragged end of a straight segment keeps to its extension at any angle, #2120; Warp Options controls verified by focused checks and original-image review; estimates unchanged; existing-guide Escape cancellation validated 2026-10-14; scores and hours unchanged; open-path anchor Delete keeps surviving runs separate; estimates unchanged; Artboard tab-switch follow-up historically validated on `a0671777`; scores and hours unchanged; Artboard clean active-tab close/Delete follow-up validated 2026-10-15; Curvature curve geometry, #1236 in part; Width-point finite-arithmetic follow-up validated; scores and hours unchanged; Image Trace adjustment discovery; later 2026-10-14 merges: macOS save sheet Format menu back to missing, #1855, #866 reopened; anchor X/Y/W/H in the Transform panel, Properties and Control bar, #1824; Snap to Grid over Snap to Pixel, #1816; toolbar labels, columns and the All Tools drawer, #1751; Actions panel selection, #1629 in part; scores unchanged; 2026-10-14 merges: anchor Align and Distribute, #1636; Delete on closed paths and whole objects, #1674; selection inside isolation, #1666; Escape in a Direct Selection drag and while typing, #1743, #1686; Pen clicks under 3 px, #1663; [ and ] on more tools, #1661 in part; fields commit on click away, #1787; command search keys, #1757; a guide's position in the Transform panel is a convenience with no parity claim, #1659; scores unchanged; intersect Smart Guides for drawn points and dragged anchors, #1602 in part; a ruler guide's position in the Transform panel, #1597 in part; Symbol Options before New Symbol, #1560; tool flyout widths, #1616; Pathfinder Trim, Merge and Crop strokes, #1570; scores unchanged; G1/M11: Artboard deletion and history selection normalization; history regressions and full workspace CI pass; scores and hours unchanged; Width Point Edit: Shift-double-click enables adjoining adjustment; plain double-click keeps it off; scores and hours unchanged; per-corner radius fields with a link toggle, split from #1594; Join merges anchors lying on one another, #1533; correction: Average and Join with a corner or smooth choice is missing, not done; guide objects take the guide colour and style and Lock Guides, #1534; Keyboard Shortcuts lists commands outside the menus, #1539; round corners next to curves and the Smooth tool's reach, #1512; Make Compound Path on a group, #1537; dialog slider rails, #1483; template layers stay in Preview in Outline view, #1486; Direct Selection takes the selected object's anchor where anchors coincide, #1508; Tabs panel stop drags, #1458; Split Into Grid dialog, #1462; Transform Objects and Reflect's angle, #1448 and #1460; Pathfinder panel on a group, #1450; value fields: readable selection, #1465 in part, and a focused field follows its value, #1457; scores unchanged; Transform dialogs have a live Preview and step with the arrow keys, #1412; transform tools, dialogs and Move act on direct-selected anchors, #1408; the bounding box hides during a corner drag, #1229; scores unchanged; dialogs focus their first field on every open, #1367, and a first field that starts greyed out once it is enabled, #1395; scores unchanged; Rectangular Grid and Polar Grid click dialogs take the skew, frame, compound path and Fill Grid options, #1377 in part, score unchanged; Live Corners: a Direct Selection click on a widget selects its corner, #1239; custom ruler origin, #1212; Artboard tool clicks fit the topmost editable object in one undo step (#1227); scores unchanged; Pathfinder single-result source names and SVG IDs retained, #1293; DXF control-point SPLINE closure read, #1299 follow-up; Document Setup: Edit Artboards closes the modal after applying valid settings; pointer-click regressions; scores and hours unchanged; Artboards: Escape exits editing; an active drag cancels first; shortcut-routing regression; scores and hours unchanged; Simplify preview/counts and valid rewritten-path selection (#1272); parity remains partial; Texture-effect drag redraws optimized (#1257); idle Windows interaction still unverified; scores unchanged; Curvature point display improved (#1236, in part); scores unchanged; Direct Selection: no-op anchor Align preserves Undo, Redo and saved state (#1180 follow-up); Eyedropper cursor distinguished from precise crosshair (#1269); scores unchanged; Eyedropper can read visible locked and outside-isolation sources (#1251); scores unchanged; Direct Selection: independent segment picks and Delete (#1278); scores unchanged; Adding paint to an unpainted zero-width stroke restores the 1 pt default (#1253); scores unchanged; macOS save sheets offer a Format menu, #866; Attributes fill-rule controls edit the compound when a member is selected, #1197; a marquee needs to touch a path, not just lie inside its fill, #1103; earlier: Live Corners round corners between curved sides, #1144) · **Target:** Adobe Illustrator 2026 (30.x; Simplify: Object › Path › Simplify opens its slider at the Auto-Simplify level for the selection, previewing that result, instead of at 1 pt; partial-path simplify remains missing; Tangent and straight-segment perpendicular snapping use saved Smart Guides Geometric Guides preferences and Control bar quick access (#2537); no View submenu; G13: gradient Method (Illustrator 30.0): a Method menu in the Gradient panel, Classic or Perceptual (`paint.editGradient`/`paint.setFill` `method`, saved in the native file only when Perceptual, so older files open as Classic); Perceptual mixes a linear or radial gradient's colours in OKLab, on the canvas and in raster exports, and SVG and PDF exports write it as an image at the Document Raster Effects resolution, as the public documentation describes; Classic draws as before; a dithered Perceptual fill takes dither's noise; score and hours unchanged; G13: gradient Dither (Illustrator 30.0): a Dither box in the Gradient panel (`paint.editGradient`/`paint.setFill` `dither`, saved in the native file) draws a linear or radial gradient fill with seeded fine noise that hides banding, on the canvas and in raster exports; SVG and PDF exports write the fill as an image at the Document Raster Effects resolution, as the public documentation describes dither; fills only, as documented; score and hours unchanged; Recolor Artwork Edit chips scroll within the available screen height (#3416); small palettes retain their tab height; scores and estimates unchanged; Blend panel Type and Ramp control step easing with a separate color Shift (#2539); Blend Options retains spacing and orientation; polynomial curves are original approximations; Panels dock into any dock group or the icon column by dragging (UI.6), workspaces keep their layout changes (UI.7), dock groups collapse to their tabs by a double-click (UI.8); scores unchanged; Shape Builder Gap Detection shares Live Paint’s spatial gap finder and temporary merge bridges (#2526); Custom is capped at 72 pt in the existing options dialog; 3D: original Standard, Diffuse, Top Left and Right lighting presets; command access, effective-setting recognition and preview/history; scores and estimates unchanged; one selected clip group's Properties panel has Edit Clipping Path and Edit Contents at its top, G1; Distribute Objects and Distribute Spacing space from a key guide (#3165) with the Distribute Spacing value; Distribute spaces selected ruler guides with the objects, #1597 in part; the selection tools take a clipping path on its outline, over the art it clips; Polygon and Star exact-size placement remembers the chosen count for later drags and dialog openings; Cancel preserves the previous count; Live trace result bounds and selection highlights exclude the retained bitmap except in source-image views; a ruler guide can be the key object Align aligns to, and selected guides align to a key object, #2143; Pencil: Alt key toggles to Smooth Tool, #2001; status-readout sample widths cached by language, document unit and pixels per point; status messages and readouts use stable responsive allocations sized for localized labels and ordinary coordinates; cached canvas dimensions match the renderer limit before cache-key creation, view construction and worker submission; cached canvas reprojection preserves scale during resize, rotation and DPI changes; on-canvas Mirror Repeat axis, #2877; Background Export progress dialog with Cancel, #2564; 3D lighting: one to ten colored lights, smooth colored Inflate, legacy white-light appearance preserved; surface batching and restored seam minimum; scores and estimates unchanged; Blend spine Anchor Point conversions, selection/stacking priority and Shift regression coverage (#2544); scores and hours unchanged; Artboard Options' X and Y with a reference point, #1876; scores unchanged; library panels: one per library, dockable and tabbable, with Persistent, #2603; the Selection tool threads text from the out port, #2572 in part; scores unchanged; Group Selection picks and moves one path of a compound path, #2441 in part; the Move dialog opens on the last move and its last checkboxes; scores unchanged; transparency grid visibility saved in native documents, #1968; Live Corners radius up to the neighbouring cuts, #2332; Image Trace quick presets: wording corrected to the shipped behaviour, the Image Trace button opens the controls; batch exporting, #1935; document window scroll bars, #2322; Anchor Point tool: a click on a handle end retracts that handle only, #2267; scores unchanged; Smart Guides: the dragged end of a straight segment keeps to its extension at any angle, #2120; Warp Options controls verified by focused checks and original-image review; estimates unchanged; existing-guide Escape cancellation validated 2026-10-14; scores and hours unchanged; open-path anchor Delete keeps surviving runs separate; estimates unchanged; Artboard tab-switch follow-up historically validated on `a0671777`; scores and hours unchanged; Artboard clean active-tab close/Delete follow-up validated 2026-10-15; Curvature curve geometry, #1236 in part; Width-point finite-arithmetic follow-up validated; scores and hours unchanged; Image Trace adjustment discovery; later 2026-10-14 merges: macOS save sheet Format menu back to missing, #1855, #866 reopened; anchor X/Y/W/H in the Transform panel, Properties and Control bar, #1824; Snap to Grid over Snap to Pixel, #1816; toolbar labels, columns and the All Tools drawer, #1751; Actions panel selection, #1629 in part; scores unchanged; 2026-10-14 merges: anchor Align and Distribute, #1636; Delete on closed paths and whole objects, #1674; selection inside isolation, #1666; Escape in a Direct Selection drag and while typing, #1743, #1686; Pen clicks under 3 px, #1663; [ and ] on more tools, #1661 in part; fields commit on click away, #1787; command search keys, #1757; a guide's position in the Transform panel is a convenience with no parity claim, #1659; scores unchanged; intersect Smart Guides for drawn points and dragged anchors, #1602 in part; a ruler guide's position in the Transform panel, #1597 in part; Symbol Options before New Symbol, #1560; tool flyout widths, #1616; Pathfinder Trim, Merge and Crop strokes, #1570; scores unchanged; G1/M11: Artboard deletion and history selection normalization; history regressions and full workspace CI pass; scores and hours unchanged; Width Point Edit: Shift-double-click enables adjoining adjustment; plain double-click keeps it off; scores and hours unchanged; per-corner radius fields with a link toggle, split from #1594; Join merges anchors lying on one another, #1533; correction: Average and Join with a corner or smooth choice is missing, not done; guide objects take the guide colour and style and Lock Guides, #1534; Keyboard Shortcuts lists commands outside the menus, #1539; round corners next to curves and the Smooth tool's reach, #1512; Make Compound Path on a group, #1537; dialog slider rails, #1483; template layers stay in Preview in Outline view, #1486; Direct Selection takes the selected object's anchor where anchors coincide, #1508; Tabs panel stop drags, #1458; Split Into Grid dialog, #1462; Transform Objects and Reflect's angle, #1448 and #1460; Pathfinder panel on a group, #1450; value fields: readable selection, #1465 in part, and a focused field follows its value, #1457; scores unchanged; Transform dialogs have a live Preview and step with the arrow keys, #1412; transform tools, dialogs and Move act on direct-selected anchors, #1408; the bounding box hides during a corner drag, #1229; scores unchanged; dialogs focus their first field on every open, #1367, and a first field that starts greyed out once it is enabled, #1395; scores unchanged; Rectangular Grid and Polar Grid click dialogs take the skew, frame, compound path and Fill Grid options, #1377 in part, score unchanged; Live Corners: a Direct Selection click on a widget selects its corner, #1239; custom ruler origin, #1212; Artboard tool clicks fit the topmost editable object in one undo step (#1227); scores unchanged; Pathfinder single-result source names and SVG IDs retained, #1293; DXF control-point SPLINE closure read, #1299 follow-up; Document Setup: Edit Artboards closes the modal after applying valid settings; pointer-click regressions; scores and hours unchanged; Artboards: Escape exits editing; an active drag cancels first; shortcut-routing regression; scores and hours unchanged; Simplify preview/counts and valid rewritten-path selection (#1272); parity remains partial; Texture-effect drag redraws optimized (#1257); idle Windows interaction still unverified; scores unchanged; Curvature point display improved (#1236, in part); scores unchanged; Direct Selection: no-op anchor Align preserves Undo, Redo and saved state (#1180 follow-up); Eyedropper cursor distinguished from precise crosshair (#1269); scores unchanged; Eyedropper can read visible locked and outside-isolation sources (#1251); scores unchanged; Direct Selection: independent segment picks and Delete (#1278); scores unchanged; Adding paint to an unpainted zero-width stroke restores the 1 pt default (#1253); scores unchanged; macOS save sheets offer a Format menu, #866; Attributes fill-rule controls edit the compound when a member is selected, #1197; a marquee needs to touch a path, not just lie inside its fill, #1103; earlier: Live Corners round corners between curved sides, #1144; interactive Mirror Repeat axis; Background Export progress dialog with Cancel, #2564; scores unchanged; 3D lighting: one to ten colored lights, smooth colored Inflate, legacy white-light appearance preserved; surface batching and restored seam minimum; scores and estimates unchanged; Blend spine Anchor Point conversions, selection/stacking priority and Shift regression coverage (#2544); scores and hours unchanged; Artboard Options' X and Y with a reference point, #1876; scores unchanged; library panels: one per library, dockable and tabbable, with Persistent, #2603; the Selection tool threads text from the out port, #2572 in part; scores unchanged; Group Selection picks and moves one path of a compound path, #2441 in part; the Move dialog opens on the last move and its last checkboxes; scores unchanged; transparency grid visibility saved in native documents, #1968; Live Corners radius up to the neighbouring cuts, #2332; Image Trace quick presets: wording corrected to the shipped behaviour, the Image Trace button opens the controls; batch exporting, #1935; document window scroll bars, #2322; Anchor Point tool: a click on a handle end retracts that handle only, #2267; scores unchanged; Smart Guides: the dragged end of a straight segment keeps to its extension at any angle, #2120; Warp Options controls verified by focused checks and original-image review; estimates unchanged; existing-guide Escape cancellation validated 2026-10-14; scores and hours unchanged; open-path anchor Delete keeps surviving runs separate; estimates unchanged; Artboard tab-switch follow-up historically validated on `a0671777`; scores and hours unchanged; Artboard clean active-tab close/Delete follow-up validated 2026-10-15; Curvature curve geometry, #1236 in part; Width-point finite-arithmetic follow-up validated; scores and hours unchanged; Image Trace adjustment discovery; later 2026-10-14 merges: macOS save sheet Format menu back to missing, #1855, #866 reopened; anchor X/Y/W/H in the Transform panel, Properties and Control bar, #1824; Snap to Grid over Snap to Pixel, #1816; toolbar labels, columns and the All Tools drawer, #1751; Actions panel selection, #1629 in part; scores unchanged; 2026-10-14 merges: anchor Align and Distribute, #1636; Delete on closed paths and whole objects, #1674; selection inside isolation, #1666; Escape in a Direct Selection drag and while typing, #1743, #1686; Pen clicks under 3 px, #1663; [ and ] on more tools, #1661 in part; fields commit on click away, #1787; command search keys, #1757; a guide's position in the Transform panel is a convenience with no parity claim, #1659; scores unchanged; intersect Smart Guides for drawn points and dragged anchors, #1602 in part; a ruler guide's position in the Transform panel, #1597 in part; Symbol Options before New Symbol, #1560; tool flyout widths, #1616; Pathfinder Trim, Merge and Crop strokes, #1570; scores unchanged; G1/M11: Artboard deletion and history selection normalization; history regressions and full workspace CI pass; scores and hours unchanged; Width Point Edit: Shift-double-click enables adjoining adjustment; plain double-click keeps it off; scores and hours unchanged; per-corner radius fields with a link toggle, split from #1594; Join merges anchors lying on one another, #1533; correction: Average and Join with a corner or smooth choice is missing, not done; guide objects take the guide colour and style and Lock Guides, #1534; Keyboard Shortcuts lists commands outside the menus, #1539; round corners next to curves and the Smooth tool's reach, #1512; Make Compound Path on a group, #1537; dialog slider rails, #1483; template layers stay in Preview in Outline view, #1486; Direct Selection takes the selected object's anchor where anchors coincide, #1508; Tabs panel stop drags, #1458; Split Into Grid dialog, #1462; Transform Objects and Reflect's angle, #1448 and #1460; Pathfinder panel on a group, #1450; value fields: readable selection, #1465 in part, and a focused field follows its value, #1457; scores unchanged; Transform dialogs have a live Preview and step with the arrow keys, #1412; transform tools, dialogs and Move act on direct-selected anchors, #1408; the bounding box hides during a corner drag, #1229; scores unchanged; dialogs focus their first field on every open, #1367, and a first field that starts greyed out once it is enabled, #1395; scores unchanged; Rectangular Grid and Polar Grid click dialogs take the skew, frame, compound path and Fill Grid options, #1377 in part, score unchanged; Live Corners: a Direct Selection click on a widget selects its corner, #1239; custom ruler origin, #1212; Artboard tool clicks fit the topmost editable object in one undo step (#1227); scores unchanged; Pathfinder single-result source names and SVG IDs retained, #1293; DXF control-point SPLINE closure read, #1299 follow-up; Document Setup: Edit Artboards closes the modal after applying valid settings; pointer-click regressions; scores and hours unchanged; Artboards: Escape exits editing; an active drag cancels first; shortcut-routing regression; scores and hours unchanged; Simplify preview/counts and valid rewritten-path selection (#1272); parity remains partial; Texture-effect drag redraws optimized (#1257); idle Windows interaction still unverified; scores unchanged; Curvature point display improved (#1236, in part); scores unchanged; Direct Selection: no-op anchor Align preserves Undo, Redo and saved state (#1180 follow-up); Eyedropper cursor distinguished from precise crosshair (#1269); scores unchanged; Eyedropper can read visible locked and outside-isolation sources (#1251); scores unchanged; Direct Selection: independent segment picks and Delete (#1278); scores unchanged; Adding paint to an unpainted zero-width stroke restores the 1 pt default (#1253); scores unchanged; macOS save sheets offer a Format menu, #866; Attributes fill-rule controls edit the compound when a member is selected, #1197; a marquee needs to touch a path, not just lie inside its fill, #1103; earlier: Live Corners round corners between curved sides, #1144) · **Target:** Adobe Illustrator 2026 (30.x; bezier drag previews follow the pointer back into the window and to the release, #834; Guides: a path moved by a press on its stroke or beside its edge, and a straight segment Direct Selection drags, snap by the nearest point of the path, landing exactly on a ruler guide (#3701); scores and estimates unchanged; previously:; Artboard Options honors Scale Artwork with Artboard for preset, orientation and typed-size changes; both Fits keep artwork stationary; scores and estimates unchanged; Tangent and straight-segment perpendicular snapping use saved Smart Guides Geometric Guides preferences and Control bar quick access (#2537); no View submenu; G13: gradient Method (Illustrator 30.0): a Method menu in the Gradient panel, Classic or Perceptual (`paint.editGradient`/`paint.setFill` `method`, saved in the native file only when Perceptual, so older files open as Classic); Perceptual mixes a linear or radial gradient's colours in OKLab, on the canvas and in raster exports, and SVG and PDF exports write it as an image at the Document Raster Effects resolution, as the public documentation describes; Classic draws as before; a dithered Perceptual fill takes dither's noise; score and hours unchanged; G13: gradient Dither (Illustrator 30.0): a Dither box in the Gradient panel (`paint.editGradient`/`paint.setFill` `dither`, saved in the native file) draws a linear or radial gradient fill with seeded fine noise that hides banding, on the canvas and in raster exports; SVG and PDF exports write the fill as an image at the Document Raster Effects resolution, as the public documentation describes dither; fills only, as documented; score and hours unchanged; Recolor Artwork Edit chips scroll within the available screen height (#3416); small palettes retain their tab height; scores and estimates unchanged; Blend panel Type and Ramp control step easing with a separate color Shift (#2539); Blend Options retains spacing and orientation; polynomial curves are original approximations; Panels dock into any dock group or the icon column by dragging (UI.6), workspaces keep their layout changes (UI.7), dock groups collapse to their tabs by a double-click (UI.8); scores unchanged; Shape Builder Gap Detection shares Live Paint’s spatial gap finder and temporary merge bridges (#2526); Custom is capped at 72 pt in the existing options dialog; 3D: original Standard, Diffuse, Top Left and Right lighting presets; command access, effective-setting recognition and preview/history; scores and estimates unchanged; one selected clip group's Properties panel has Edit Clipping Path and Edit Contents at its top, G1; Distribute Objects and Distribute Spacing space from a key guide (#3165) with the Distribute Spacing value; Distribute spaces selected ruler guides with the objects, #1597 in part; the selection tools take a clipping path on its outline, over the art it clips; Polygon and Star exact-size placement remembers the chosen count for later drags and dialog openings; Cancel preserves the previous count; Live trace result bounds and selection highlights exclude the retained bitmap except in source-image views; a ruler guide can be the key object Align aligns to, and selected guides align to a key object, #2143; Pencil: Alt key toggles to Smooth Tool, #2001; status-readout sample widths cached by language, document unit and pixels per point; status messages and readouts use stable responsive allocations sized for localized labels and ordinary coordinates; cached canvas dimensions match the renderer limit before cache-key creation, view construction and worker submission; cached canvas reprojection preserves scale during resize, rotation and DPI changes; on-canvas Mirror Repeat axis, #2877; Background Export progress dialog with Cancel, #2564; 3D lighting: one to ten colored lights, smooth colored Inflate, legacy white-light appearance preserved; surface batching and restored seam minimum; scores and estimates unchanged; Blend spine Anchor Point conversions, selection/stacking priority and Shift regression coverage (#2544); scores and hours unchanged; Artboard Options' X and Y with a reference point, #1876; scores unchanged; library panels: one per library, dockable and tabbable, with Persistent, #2603; the Selection tool threads text from the out port, #2572 in part; scores unchanged; Group Selection picks and moves one path of a compound path, #2441 in part; the Move dialog opens on the last move and its last checkboxes; scores unchanged; transparency grid visibility saved in native documents, #1968; Live Corners radius up to the neighbouring cuts, #2332; Image Trace quick presets: wording corrected to the shipped behaviour, the Image Trace button opens the controls; batch exporting, #1935; document window scroll bars, #2322; Anchor Point tool: a click on a handle end retracts that handle only, #2267; scores unchanged; Smart Guides: the dragged end of a straight segment keeps to its extension at any angle, #2120; Warp Options controls verified by focused checks and original-image review; estimates unchanged; existing-guide Escape cancellation validated 2026-10-14; scores and hours unchanged; open-path anchor Delete keeps surviving runs separate; estimates unchanged; Artboard tab-switch follow-up historically validated on `a0671777`; scores and hours unchanged; Artboard clean active-tab close/Delete follow-up validated 2026-10-15; Curvature curve geometry, #1236 in part; Width-point finite-arithmetic follow-up validated; scores and hours unchanged; Image Trace adjustment discovery; later 2026-10-14 merges: macOS save sheet Format menu back to missing, #1855, #866 reopened; anchor X/Y/W/H in the Transform panel, Properties and Control bar, #1824; Snap to Grid over Snap to Pixel, #1816; toolbar labels, columns and the All Tools drawer, #1751; Actions panel selection, #1629 in part; scores unchanged; 2026-10-14 merges: anchor Align and Distribute, #1636; Delete on closed paths and whole objects, #1674; selection inside isolation, #1666; Escape in a Direct Selection drag and while typing, #1743, #1686; Pen clicks under 3 px, #1663; [ and ] on more tools, #1661 in part; fields commit on click away, #1787; command search keys, #1757; a guide's position in the Transform panel is a convenience with no parity claim, #1659; scores unchanged; intersect Smart Guides for drawn points and dragged anchors, #1602 in part; a ruler guide's position in the Transform panel, #1597 in part; Symbol Options before New Symbol, #1560; tool flyout widths, #1616; Pathfinder Trim, Merge and Crop strokes, #1570; scores unchanged; G1/M11: Artboard deletion and history selection normalization; history regressions and full workspace CI pass; scores and hours unchanged; Width Point Edit: Shift-double-click enables adjoining adjustment; plain double-click keeps it off; scores and hours unchanged; per-corner radius fields with a link toggle, split from #1594; Join merges anchors lying on one another, #1533; correction: Average and Join with a corner or smooth choice is missing, not done; guide objects take the guide colour and style and Lock Guides, #1534; Keyboard Shortcuts lists commands outside the menus, #1539; round corners next to curves and the Smooth tool's reach, #1512; Make Compound Path on a group, #1537; dialog slider rails, #1483; template layers stay in Preview in Outline view, #1486; Direct Selection takes the selected object's anchor where anchors coincide, #1508; Tabs panel stop drags, #1458; Split Into Grid dialog, #1462; Transform Objects and Reflect's angle, #1448 and #1460; Pathfinder panel on a group, #1450; value fields: readable selection, #1465 in part, and a focused field follows its value, #1457; scores unchanged; Transform dialogs have a live Preview and step with the arrow keys, #1412; transform tools, dialogs and Move act on direct-selected anchors, #1408; the bounding box hides during a corner drag, #1229; scores unchanged; dialogs focus their first field on every open, #1367, and a first field that starts greyed out once it is enabled, #1395; scores unchanged; Rectangular Grid and Polar Grid click dialogs take the skew, frame, compound path and Fill Grid options, #1377 in part, score unchanged; Live Corners: a Direct Selection click on a widget selects its corner, #1239; custom ruler origin, #1212; Artboard tool clicks fit the topmost editable object in one undo step (#1227); scores unchanged; Pathfinder single-result source names and SVG IDs retained, #1293; DXF control-point SPLINE closure read, #1299 follow-up; Document Setup: Edit Artboards closes the modal after applying valid settings; pointer-click regressions; scores and hours unchanged; Artboards: Escape exits editing; an active drag cancels first; shortcut-routing regression; scores and hours unchanged; Simplify preview/counts and valid rewritten-path selection (#1272); parity remains partial; Texture-effect drag redraws optimized (#1257); idle Windows interaction still unverified; scores unchanged; Curvature point display improved (#1236, in part); scores unchanged; Direct Selection: no-op anchor Align preserves Undo, Redo and saved state (#1180 follow-up); Eyedropper cursor distinguished from precise crosshair (#1269); scores unchanged; Eyedropper can read visible locked and outside-isolation sources (#1251); scores unchanged; Direct Selection: independent segment picks and Delete (#1278); scores unchanged; Adding paint to an unpainted zero-width stroke restores the 1 pt default (#1253); scores unchanged; macOS save sheets offer a Format menu, #866; Attributes fill-rule controls edit the compound when a member is selected, #1197; a marquee needs to touch a path, not just lie inside its fill, #1103; earlier: Live Corners round corners between curved sides, #1144; interactive Mirror Repeat axis; Background Export progress dialog with Cancel, #2564; scores unchanged; 3D lighting: one to ten colored lights, smooth colored Inflate, legacy white-light appearance preserved; surface batching and restored seam minimum; scores and estimates unchanged; Blend spine Anchor Point conversions, selection/stacking priority and Shift regression coverage (#2544); scores and hours unchanged; Artboard Options' X and Y with a reference point, #1876; scores unchanged; library panels: one per library, dockable and tabbable, with Persistent, #2603; the Selection tool threads text from the out port, #2572 in part; scores unchanged; Group Selection picks and moves one path of a compound path, #2441 in part; the Move dialog opens on the last move and its last checkboxes; scores unchanged; transparency grid visibility saved in native documents, #1968; Live Corners radius up to the neighbouring cuts, #2332; Image Trace quick presets: wording corrected to the shipped behaviour, the Image Trace button opens the controls; batch exporting, #1935; document window scroll bars, #2322; Anchor Point tool: a click on a handle end retracts that handle only, #2267; scores unchanged; Smart Guides: the dragged end of a straight segment keeps to its extension at any angle, #2120; Warp Options controls verified by focused checks and original-image review; estimates unchanged; existing-guide Escape cancellation validated 2026-10-14; scores and hours unchanged; open-path anchor Delete keeps surviving runs separate; estimates unchanged; Artboard tab-switch follow-up historically validated on `a0671777`; scores and hours unchanged; Artboard clean active-tab close/Delete follow-up validated 2026-10-15; Curvature curve geometry, #1236 in part; Width-point finite-arithmetic follow-up validated; scores and hours unchanged; Image Trace adjustment discovery; later 2026-10-14 merges: macOS save sheet Format menu back to missing, #1855, #866 reopened; anchor X/Y/W/H in the Transform panel, Properties and Control bar, #1824; Snap to Grid over Snap to Pixel, #1816; toolbar labels, columns and the All Tools drawer, #1751; Actions panel selection, #1629 in part; scores unchanged; 2026-10-14 merges: anchor Align and Distribute, #1636; Delete on closed paths and whole objects, #1674; selection inside isolation, #1666; Escape in a Direct Selection drag and while typing, #1743, #1686; Pen clicks under 3 px, #1663; [ and ] on more tools, #1661 in part; fields commit on click away, #1787; command search keys, #1757; a guide's position in the Transform panel is a convenience with no parity claim, #1659; scores unchanged; intersect Smart Guides for drawn points and dragged anchors, #1602 in part; a ruler guide's position in the Transform panel, #1597 in part; Symbol Options before New Symbol, #1560; tool flyout widths, #1616; Pathfinder Trim, Merge and Crop strokes, #1570; scores unchanged; G1/M11: Artboard deletion and history selection normalization; history regressions and full workspace CI pass; scores and hours unchanged; Width Point Edit: Shift-double-click enables adjoining adjustment; plain double-click keeps it off; scores and hours unchanged; per-corner radius fields with a link toggle, split from #1594; Join merges anchors lying on one another, #1533; correction: Average and Join with a corner or smooth choice is missing, not done; guide objects take the guide colour and style and Lock Guides, #1534; Keyboard Shortcuts lists commands outside the menus, #1539; round corners next to curves and the Smooth tool's reach, #1512; Make Compound Path on a group, #1537; dialog slider rails, #1483; template layers stay in Preview in Outline view, #1486; Direct Selection takes the selected object's anchor where anchors coincide, #1508; Tabs panel stop drags, #1458; Split Into Grid dialog, #1462; Transform Objects and Reflect's angle, #1448 and #1460; Pathfinder panel on a group, #1450; value fields: readable selection, #1465 in part, and a focused field follows its value, #1457; scores unchanged; Transform dialogs have a live Preview and step with the arrow keys, #1412; transform tools, dialogs and Move act on direct-selected anchors, #1408; the bounding box hides during a corner drag, #1229; scores unchanged; dialogs focus their first field on every open, #1367, and a first field that starts greyed out once it is enabled, #1395; scores unchanged; Rectangular Grid and Polar Grid click dialogs take the skew, frame, compound path and Fill Grid options, #1377 in part, score unchanged; Live Corners: a Direct Selection click on a widget selects its corner, #1239; custom ruler origin, #1212; Artboard tool clicks fit the topmost editable object in one undo step (#1227); scores unchanged; Pathfinder single-result source names and SVG IDs retained, #1293; DXF control-point SPLINE closure read, #1299 follow-up; Document Setup: Edit Artboards closes the modal after applying valid settings; pointer-click regressions; scores and hours unchanged; Artboards: Escape exits editing; an active drag cancels first; shortcut-routing regression; scores and hours unchanged; Simplify preview/counts and valid rewritten-path selection (#1272); parity remains partial; Texture-effect drag redraws optimized (#1257); idle Windows interaction still unverified; scores unchanged; Curvature point display improved (#1236, in part); scores unchanged; Direct Selection: no-op anchor Align preserves Undo, Redo and saved state (#1180 follow-up); Eyedropper cursor distinguished from precise crosshair (#1269); scores unchanged; Eyedropper can read visible locked and outside-isolation sources (#1251); scores unchanged; Direct Selection: independent segment picks and Delete (#1278); scores unchanged; Adding paint to an unpainted zero-width stroke restores the 1 pt default (#1253); scores unchanged; macOS save sheets offer a Format menu, #866; Attributes fill-rule controls edit the compound when a member is selected, #1197; a marquee needs to touch a path, not just lie inside its fill, #1103; earlier: Live Corners round corners between curved sides, #1144) · **Target:** Adobe Illustrator 2026 (30.x; Preferences › General › Constrain Angle: arrow keys, Shift drags and new objects follow it, #394; bezier drag previews follow the pointer back into the window and to the release, #834; Guides: a path moved by a press on its stroke or beside its edge, and a straight segment Direct Selection drags, snap by the nearest point of the path, landing exactly on a ruler guide (#3701); scores and estimates unchanged; previously:; Artboard Options honors Scale Artwork with Artboard for preset, orientation and typed-size changes; both Fits keep artwork stationary; scores and estimates unchanged; Tangent and straight-segment perpendicular snapping use saved Smart Guides Geometric Guides preferences and Control bar quick access (#2537); no View submenu; G13: gradient Method (Illustrator 30.0): a Method menu in the Gradient panel, Classic or Perceptual (`paint.editGradient`/`paint.setFill` `method`, saved in the native file only when Perceptual, so older files open as Classic); Perceptual mixes a linear or radial gradient's colours in OKLab, on the canvas and in raster exports, and SVG and PDF exports write it as an image at the Document Raster Effects resolution, as the public documentation describes; Classic draws as before; a dithered Perceptual fill takes dither's noise; score and hours unchanged; G13: gradient Dither (Illustrator 30.0): a Dither box in the Gradient panel (`paint.editGradient`/`paint.setFill` `dither`, saved in the native file) draws a linear or radial gradient fill with seeded fine noise that hides banding, on the canvas and in raster exports; SVG and PDF exports write the fill as an image at the Document Raster Effects resolution, as the public documentation describes dither; fills only, as documented; score and hours unchanged; Recolor Artwork Edit chips scroll within the available screen height (#3416); small palettes retain their tab height; scores and estimates unchanged; Blend panel Type and Ramp control step easing with a separate color Shift (#2539); Blend Options retains spacing and orientation; polynomial curves are original approximations; Panels dock into any dock group or the icon column by dragging (UI.6), workspaces keep their layout changes (UI.7), dock groups collapse to their tabs by a double-click (UI.8); scores unchanged; Shape Builder Gap Detection shares Live Paint’s spatial gap finder and temporary merge bridges (#2526); Custom is capped at 72 pt in the existing options dialog; 3D: original Standard, Diffuse, Top Left and Right lighting presets; command access, effective-setting recognition and preview/history; scores and estimates unchanged; one selected clip group's Properties panel has Edit Clipping Path and Edit Contents at its top, G1; Distribute Objects and Distribute Spacing space from a key guide (#3165) with the Distribute Spacing value; Distribute spaces selected ruler guides with the objects, #1597 in part; the selection tools take a clipping path on its outline, over the art it clips; Polygon and Star exact-size placement remembers the chosen count for later drags and dialog openings; Cancel preserves the previous count; Live trace result bounds and selection highlights exclude the retained bitmap except in source-image views; a ruler guide can be the key object Align aligns to, and selected guides align to a key object, #2143; Pencil: Alt key toggles to Smooth Tool, #2001; status-readout sample widths cached by language, document unit and pixels per point; status messages and readouts use stable responsive allocations sized for localized labels and ordinary coordinates; cached canvas dimensions match the renderer limit before cache-key creation, view construction and worker submission; cached canvas reprojection preserves scale during resize, rotation and DPI changes; on-canvas Mirror Repeat axis, #2877; Background Export progress dialog with Cancel, #2564; 3D lighting: one to ten colored lights, smooth colored Inflate, legacy white-light appearance preserved; surface batching and restored seam minimum; scores and estimates unchanged; Blend spine Anchor Point conversions, selection/stacking priority and Shift regression coverage (#2544); scores and hours unchanged; Artboard Options' X and Y with a reference point, #1876; scores unchanged; library panels: one per library, dockable and tabbable, with Persistent, #2603; the Selection tool threads text from the out port, #2572 in part; scores unchanged; Group Selection picks and moves one path of a compound path, #2441 in part; the Move dialog opens on the last move and its last checkboxes; scores unchanged; transparency grid visibility saved in native documents, #1968; Live Corners radius up to the neighbouring cuts, #2332; Image Trace quick presets: wording corrected to the shipped behaviour, the Image Trace button opens the controls; batch exporting, #1935; document window scroll bars, #2322; Anchor Point tool: a click on a handle end retracts that handle only, #2267; scores unchanged; Smart Guides: the dragged end of a straight segment keeps to its extension at any angle, #2120; Warp Options controls verified by focused checks and original-image review; estimates unchanged; existing-guide Escape cancellation validated 2026-10-14; scores and hours unchanged; open-path anchor Delete keeps surviving runs separate; estimates unchanged; Artboard tab-switch follow-up historically validated on `a0671777`; scores and hours unchanged; Artboard clean active-tab close/Delete follow-up validated 2026-10-15; Curvature curve geometry, #1236 in part; Width-point finite-arithmetic follow-up validated; scores and hours unchanged; Image Trace adjustment discovery; later 2026-10-14 merges: macOS save sheet Format menu back to missing, #1855, #866 reopened; anchor X/Y/W/H in the Transform panel, Properties and Control bar, #1824; Snap to Grid over Snap to Pixel, #1816; toolbar labels, columns and the All Tools drawer, #1751; Actions panel selection, #1629 in part; scores unchanged; 2026-10-14 merges: anchor Align and Distribute, #1636; Delete on closed paths and whole objects, #1674; selection inside isolation, #1666; Escape in a Direct Selection drag and while typing, #1743, #1686; Pen clicks under 3 px, #1663; [ and ] on more tools, #1661 in part; fields commit on click away, #1787; command search keys, #1757; a guide's position in the Transform panel is a convenience with no parity claim, #1659; scores unchanged; intersect Smart Guides for drawn points and dragged anchors, #1602 in part; a ruler guide's position in the Transform panel, #1597 in part; Symbol Options before New Symbol, #1560; tool flyout widths, #1616; Pathfinder Trim, Merge and Crop strokes, #1570; scores unchanged; G1/M11: Artboard deletion and history selection normalization; history regressions and full workspace CI pass; scores and hours unchanged; Width Point Edit: Shift-double-click enables adjoining adjustment; plain double-click keeps it off; scores and hours unchanged; per-corner radius fields with a link toggle, split from #1594; Join merges anchors lying on one another, #1533; correction: Average and Join with a corner or smooth choice is missing, not done; guide objects take the guide colour and style and Lock Guides, #1534; Keyboard Shortcuts lists commands outside the menus, #1539; round corners next to curves and the Smooth tool's reach, #1512; Make Compound Path on a group, #1537; dialog slider rails, #1483; template layers stay in Preview in Outline view, #1486; Direct Selection takes the selected object's anchor where anchors coincide, #1508; Tabs panel stop drags, #1458; Split Into Grid dialog, #1462; Transform Objects and Reflect's angle, #1448 and #1460; Pathfinder panel on a group, #1450; value fields: readable selection, #1465 in part, and a focused field follows its value, #1457; scores unchanged; Transform dialogs have a live Preview and step with the arrow keys, #1412; transform tools, dialogs and Move act on direct-selected anchors, #1408; the bounding box hides during a corner drag, #1229; scores unchanged; dialogs focus their first field on every open, #1367, and a first field that starts greyed out once it is enabled, #1395; scores unchanged; Rectangular Grid and Polar Grid click dialogs take the skew, frame, compound path and Fill Grid options, #1377 in part, score unchanged; Live Corners: a Direct Selection click on a widget selects its corner, #1239; custom ruler origin, #1212; Artboard tool clicks fit the topmost editable object in one undo step (#1227); scores unchanged; Pathfinder single-result source names and SVG IDs retained, #1293; DXF control-point SPLINE closure read, #1299 follow-up; Document Setup: Edit Artboards closes the modal after applying valid settings; pointer-click regressions; scores and hours unchanged; Artboards: Escape exits editing; an active drag cancels first; shortcut-routing regression; scores and hours unchanged; Simplify preview/counts and valid rewritten-path selection (#1272); parity remains partial; Texture-effect drag redraws optimized (#1257); idle Windows interaction still unverified; scores unchanged; Curvature point display improved (#1236, in part); scores unchanged; Direct Selection: no-op anchor Align preserves Undo, Redo and saved state (#1180 follow-up); Eyedropper cursor distinguished from precise crosshair (#1269); scores unchanged; Eyedropper can read visible locked and outside-isolation sources (#1251); scores unchanged; Direct Selection: independent segment picks and Delete (#1278); scores unchanged; Adding paint to an unpainted zero-width stroke restores the 1 pt default (#1253); scores unchanged; macOS save sheets offer a Format menu, #866; Attributes fill-rule controls edit the compound when a member is selected, #1197; a marquee needs to touch a path, not just lie inside its fill, #1103; earlier: Live Corners round corners between curved sides, #1144; interactive Mirror Repeat axis; Background Export progress dialog with Cancel, #2564; scores unchanged; 3D lighting: one to ten colored lights, smooth colored Inflate, legacy white-light appearance preserved; surface batching and restored seam minimum; scores and estimates unchanged; Blend spine Anchor Point conversions, selection/stacking priority and Shift regression coverage (#2544); scores and hours unchanged; Artboard Options' X and Y with a reference point, #1876; scores unchanged; library panels: one per library, dockable and tabbable, with Persistent, #2603; the Selection tool threads text from the out port, #2572 in part; scores unchanged; Group Selection picks and moves one path of a compound path, #2441 in part; the Move dialog opens on the last move and its last checkboxes; scores unchanged; transparency grid visibility saved in native documents, #1968; Live Corners radius up to the neighbouring cuts, #2332; Image Trace quick presets: wording corrected to the shipped behaviour, the Image Trace button opens the controls; batch exporting, #1935; document window scroll bars, #2322; Anchor Point tool: a click on a handle end retracts that handle only, #2267; scores unchanged; Smart Guides: the dragged end of a straight segment keeps to its extension at any angle, #2120; Warp Options controls verified by focused checks and original-image review; estimates unchanged; existing-guide Escape cancellation validated 2026-10-14; scores and hours unchanged; open-path anchor Delete keeps surviving runs separate; estimates unchanged; Artboard tab-switch follow-up historically validated on `a0671777`; scores and hours unchanged; Artboard clean active-tab close/Delete follow-up validated 2026-10-15; Curvature curve geometry, #1236 in part; Width-point finite-arithmetic follow-up validated; scores and hours unchanged; Image Trace adjustment discovery; later 2026-10-14 merges: macOS save sheet Format menu back to missing, #1855, #866 reopened; anchor X/Y/W/H in the Transform panel, Properties and Control bar, #1824; Snap to Grid over Snap to Pixel, #1816; toolbar labels, columns and the All Tools drawer, #1751; Actions panel selection, #1629 in part; scores unchanged; 2026-10-14 merges: anchor Align and Distribute, #1636; Delete on closed paths and whole objects, #1674; selection inside isolation, #1666; Escape in a Direct Selection drag and while typing, #1743, #1686; Pen clicks under 3 px, #1663; [ and ] on more tools, #1661 in part; fields commit on click away, #1787; command search keys, #1757; a guide's position in the Transform panel is a convenience with no parity claim, #1659; scores unchanged; intersect Smart Guides for drawn points and dragged anchors, #1602 in part; a ruler guide's position in the Transform panel, #1597 in part; Symbol Options before New Symbol, #1560; tool flyout widths, #1616; Pathfinder Trim, Merge and Crop strokes, #1570; scores unchanged; G1/M11: Artboard deletion and history selection normalization; history regressions and full workspace CI pass; scores and hours unchanged; Width Point Edit: Shift-double-click enables adjoining adjustment; plain double-click keeps it off; scores and hours unchanged; per-corner radius fields with a link toggle, split from #1594; Join merges anchors lying on one another, #1533; correction: Average and Join with a corner or smooth choice is missing, not done; guide objects take the guide colour and style and Lock Guides, #1534; Keyboard Shortcuts lists commands outside the menus, #1539; round corners next to curves and the Smooth tool's reach, #1512; Make Compound Path on a group, #1537; dialog slider rails, #1483; template layers stay in Preview in Outline view, #1486; Direct Selection takes the selected object's anchor where anchors coincide, #1508; Tabs panel stop drags, #1458; Split Into Grid dialog, #1462; Transform Objects and Reflect's angle, #1448 and #1460; Pathfinder panel on a group, #1450; value fields: readable selection, #1465 in part, and a focused field follows its value, #1457; scores unchanged; Transform dialogs have a live Preview and step with the arrow keys, #1412; transform tools, dialogs and Move act on direct-selected anchors, #1408; the bounding box hides during a corner drag, #1229; scores unchanged; dialogs focus their first field on every open, #1367, and a first field that starts greyed out once it is enabled, #1395; scores unchanged; Rectangular Grid and Polar Grid click dialogs take the skew, frame, compound path and Fill Grid options, #1377 in part, score unchanged; Live Corners: a Direct Selection click on a widget selects its corner, #1239; custom ruler origin, #1212; Artboard tool clicks fit the topmost editable object in one undo step (#1227); scores unchanged; Pathfinder single-result source names and SVG IDs retained, #1293; DXF control-point SPLINE closure read, #1299 follow-up; Document Setup: Edit Artboards closes the modal after applying valid settings; pointer-click regressions; scores and hours unchanged; Artboards: Escape exits editing; an active drag cancels first; shortcut-routing regression; scores and hours unchanged; Simplify preview/counts and valid rewritten-path selection (#1272); parity remains partial; Texture-effect drag redraws optimized (#1257); idle Windows interaction still unverified; scores unchanged; Curvature point display improved (#1236, in part); scores unchanged; Direct Selection: no-op anchor Align preserves Undo, Redo and saved state (#1180 follow-up); Eyedropper cursor distinguished from precise crosshair (#1269); scores unchanged; Eyedropper can read visible locked and outside-isolation sources (#1251); scores unchanged; Direct Selection: independent segment picks and Delete (#1278); scores unchanged; Adding paint to an unpainted zero-width stroke restores the 1 pt default (#1253); scores unchanged; macOS save sheets offer a Format menu, #866; Attributes fill-rule controls edit the compound when a member is selected, #1197; a marquee needs to touch a path, not just lie inside its fill, #1103; earlier: Live Corners round corners between curved sides, #1144) · **Target:** Adobe Illustrator 2026 (30.x; Merge Live Paint in Properties only for what Live Paint can take; Properties has the overview's contextual controls (Blend Options, Merge Live Paint, Width Profile, an effect's Delete icon, Direct-Selected corners), G1; previously: Select › Start/Stop Global Edit with the Global Edit Options in Properties › Quick Actions, Escape stops it, #3521; previously:; Window › Symbol Libraries and the Symbols panel's Symbol Libraries Menu open symbol library panels; symbols click to add and drag onto the canvas; previously:; Selection tool: in and out ports on every selected area type and type on a path, the in port threading a new or clicked frame before it, #2572; scores unchanged; Smart guides: Distance Guides, Alt/Option over another object or an empty part of an artboard with the Selection tool, and the Invoke Distance Guides preference; scores and estimates unchanged; previously:; Artboard Options: Show Video Safe Areas (`showVideoSafeAreas`) outlines the action-safe and title-safe areas 3.5% and 5% in from each edge (EBU R 95) on the canvas only, and Video Ruler Pixel Aspect Ratio (`videoRulerPixelAspectRatio`, 0.1–10, default 1) sets how wide a video pixel is, per artboard, in one undo step and in the native format; View › Rulers › Show Video Rulers (`view.videoRulers`) counts the active artboard in those pixels from its top-left corner (#2849); scores and estimates unchanged; a double-click on the Line Segment, Arc, Rectangular Grid or Polar Grid tool opens its Tool Options, whose OK sets the next drag's and click's options (#2691, in part); scores and estimates unchanged; Pencil: Shift held at the press draws a straight line at 45° steps from the Constrain Angle (part of #3799); status-bar pointer readout follow-up; scores and hours unchanged; Simplify of part of a path: with anchors or segments of a path direct-selected, Object › Path › Simplify (slider, Auto-Simplify and dialog) refits only each run of selected segments between its end anchors and leaves the rest of the path exactly as it was; the Simplify row is done; scores and estimates unchanged; Fix #3705: in Draw Inside, art drawn or pasted inside the isolated object takes isolation mode into its clip group, so it can be clicked and isn't dimmed; undo or redo that removes the isolated object ends isolation mode; Fix #3705 in part: drawing modes apply to pastes (Draw Inside puts Paste, Paste in Place and Paste on All Artboards inside the object drawn into; Draw Behind puts Paste in Place and Paste on All Artboards behind the selection), and Shift+D goes from Draw Behind back to Draw Normal when Draw Inside can't draw into the selection instead of stopping at Draw Behind; Tangent and straight-segment perpendicular snapping use saved Smart Guides Geometric Guides preferences and Control bar quick access (#2537); no View submenu; G13: gradient Method (Illustrator 30.0): a Method menu in the Gradient panel, Classic or Perceptual (`paint.editGradient`/`paint.setFill` `method`, saved in the native file only when Perceptual, so older files open as Classic); Perceptual mixes a linear or radial gradient's colours in OKLab, on the canvas and in raster exports, and SVG and PDF exports write it as an image at the Document Raster Effects resolution, as the public documentation describes; Classic draws as before; a dithered Perceptual fill takes dither's noise; score and hours unchanged; G13: gradient Dither (Illustrator 30.0): a Dither box in the Gradient panel (`paint.editGradient`/`paint.setFill` `dither`, saved in the native file) draws a linear or radial gradient fill with seeded fine noise that hides banding, on the canvas and in raster exports; SVG and PDF exports write the fill as an image at the Document Raster Effects resolution, as the public documentation describes dither; fills only, as documented; score and hours unchanged; Recolor Artwork Edit chips scroll within the available screen height (#3416); small palettes retain their tab height; scores and estimates unchanged; Blend panel Type and Ramp control step easing with a separate color Shift (#2539); Blend Options retains spacing and orientation; polynomial curves are original approximations; Panels dock into any dock group or the icon column by dragging (UI.6), workspaces keep their layout changes (UI.7), dock groups collapse to their tabs by a double-click (UI.8); scores unchanged; Shape Builder Gap Detection shares Live Paint’s spatial gap finder and temporary merge bridges (#2526); Custom is capped at 72 pt in the existing options dialog; 3D: original Standard, Diffuse, Top Left and Right lighting presets; command access, effective-setting recognition and preview/history; scores and estimates unchanged; one selected clip group's Properties panel has Edit Clipping Path and Edit Contents at its top, G1; Distribute Objects and Distribute Spacing space from a key guide (#3165) with the Distribute Spacing value; Distribute spaces selected ruler guides with the objects, #1597 in part; the selection tools take a clipping path on its outline, over the art it clips; Polygon and Star exact-size placement remembers the chosen count for later drags and dialog openings; Cancel preserves the previous count; Live trace result bounds and selection highlights exclude the retained bitmap except in source-image views; a ruler guide can be the key object Align aligns to, and selected guides align to a key object, #2143; Pencil: Alt key toggles to Smooth Tool, #2001; status-readout sample widths cached by language, document unit and pixels per point; status messages and readouts use stable responsive allocations sized for localized labels and ordinary coordinates; cached canvas dimensions match the renderer limit before cache-key creation, view construction and worker submission; cached canvas reprojection preserves scale during resize, rotation and DPI changes; on-canvas Mirror Repeat axis, #2877; Background Export progress dialog with Cancel, #2564; 3D lighting: one to ten colored lights, smooth colored Inflate, legacy white-light appearance preserved; surface batching and restored seam minimum; scores and estimates unchanged; Blend spine Anchor Point conversions, selection/stacking priority and Shift regression coverage (#2544); scores and hours unchanged; Artboard Options' X and Y with a reference point, #1876; scores unchanged; library panels: one per library, dockable and tabbable, with Persistent, #2603; the Selection tool threads text from the out port, #2572 in part; scores unchanged; Group Selection picks and moves one path of a compound path, #2441 in part; the Move dialog opens on the last move and its last checkboxes; scores unchanged; transparency grid visibility saved in native documents, #1968; Live Corners radius up to the neighbouring cuts, #2332; Image Trace quick presets: wording corrected to the shipped behaviour, the Image Trace button opens the controls; batch exporting, #1935; document window scroll bars, #2322; Anchor Point tool: a click on a handle end retracts that handle only, #2267; scores unchanged; Smart Guides: the dragged end of a straight segment keeps to its extension at any angle, #2120; Warp Options controls verified by focused checks and original-image review; estimates unchanged; existing-guide Escape cancellation validated 2026-10-14; scores and hours unchanged; open-path anchor Delete keeps surviving runs separate; estimates unchanged; Artboard tab-switch follow-up historically validated on `a0671777`; scores and hours unchanged; Artboard clean active-tab close/Delete follow-up validated 2026-10-15; Curvature curve geometry, #1236 in part; Width-point finite-arithmetic follow-up validated; scores and hours unchanged; Image Trace adjustment discovery; later 2026-10-14 merges: macOS save sheet Format menu back to missing, #1855, #866 reopened; anchor X/Y/W/H in the Transform panel, Properties and Control bar, #1824; Snap to Grid over Snap to Pixel, #1816; toolbar labels, columns and the All Tools drawer, #1751; Actions panel selection, #1629 in part; scores unchanged; 2026-10-14 merges: anchor Align and Distribute, #1636; Delete on closed paths and whole objects, #1674; selection inside isolation, #1666; Escape in a Direct Selection drag and while typing, #1743, #1686; Pen clicks under 3 px, #1663; [ and ] on more tools, #1661 in part; fields commit on click away, #1787; command search keys, #1757; a guide's position in the Transform panel is a convenience with no parity claim, #1659; scores unchanged; intersect Smart Guides for drawn points and dragged anchors, #1602 in part; a ruler guide's position in the Transform panel, #1597 in part; Symbol Options before New Symbol, #1560; tool flyout widths, #1616; Pathfinder Trim, Merge and Crop strokes, #1570; scores unchanged; G1/M11: Artboard deletion and history selection normalization; history regressions and full workspace CI pass; scores and hours unchanged; Width Point Edit: Shift-double-click enables adjoining adjustment; plain double-click keeps it off; scores and hours unchanged; per-corner radius fields with a link toggle, split from #1594; Join merges anchors lying on one another, #1533; correction: Average and Join with a corner or smooth choice is missing, not done; guide objects take the guide colour and style and Lock Guides, #1534; Keyboard Shortcuts lists commands outside the menus, #1539; round corners next to curves and the Smooth tool's reach, #1512; Make Compound Path on a group, #1537; dialog slider rails, #1483; template layers stay in Preview in Outline view, #1486; Direct Selection takes the selected object's anchor where anchors coincide, #1508; Tabs panel stop drags, #1458; Split Into Grid dialog, #1462; Transform Objects and Reflect's angle, #1448 and #1460; Pathfinder panel on a group, #1450; value fields: readable selection, #1465 in part, and a focused field follows its value, #1457; scores unchanged; Transform dialogs have a live Preview and step with the arrow keys, #1412; transform tools, dialogs and Move act on direct-selected anchors, #1408; the bounding box hides during a corner drag, #1229; scores unchanged; dialogs focus their first field on every open, #1367, and a first field that starts greyed out once it is enabled, #1395; scores unchanged; Rectangular Grid and Polar Grid click dialogs take the skew, frame, compound path and Fill Grid options, #1377 in part, score unchanged; Live Corners: a Direct Selection click on a widget selects its corner, #1239; custom ruler origin, #1212; Artboard tool clicks fit the topmost editable object in one undo step (#1227); scores unchanged; Pathfinder single-result source names and SVG IDs retained, #1293; DXF control-point SPLINE closure read, #1299 follow-up; Document Setup: Edit Artboards closes the modal after applying valid settings; pointer-click regressions; scores and hours unchanged; Artboards: Escape exits editing; an active drag cancels first; shortcut-routing regression; scores and hours unchanged; Simplify preview/counts and valid rewritten-path selection (#1272); parity remains partial; Texture-effect drag redraws optimized (#1257); idle Windows interaction still unverified; scores unchanged; Curvature point display improved (#1236, in part); scores unchanged; Direct Selection: no-op anchor Align preserves Undo, Redo and saved state (#1180 follow-up); Eyedropper cursor distinguished from precise crosshair (#1269); scores unchanged; Eyedropper can read visible locked and outside-isolation sources (#1251); scores unchanged; Direct Selection: independent segment picks and Delete (#1278); scores unchanged; Adding paint to an unpainted zero-width stroke restores the 1 pt default (#1253); scores unchanged; macOS save sheets offer a Format menu, #866; Attributes fill-rule controls edit the compound when a member is selected, #1197; a marquee needs to touch a path, not just lie inside its fill, #1103; earlier: Live Corners round corners between curved sides, #1144; interactive Mirror Repeat axis; Background Export progress dialog with Cancel, #2564; scores unchanged; 3D lighting: one to ten colored lights, smooth colored Inflate, legacy white-light appearance preserved; surface batching and restored seam minimum; scores and estimates unchanged; Blend spine Anchor Point conversions, selection/stacking priority and Shift regression coverage (#2544); scores and hours unchanged; Artboard Options' X and Y with a reference point, #1876; scores unchanged; library panels: one per library, dockable and tabbable, with Persistent, #2603; the Selection tool threads text from the out port, #2572 in part; scores unchanged; Group Selection picks and moves one path of a compound path, #2441 in part; the Move dialog opens on the last move and its last checkboxes; scores unchanged; transparency grid visibility saved in native documents, #1968; Live Corners radius up to the neighbouring cuts, #2332; Image Trace quick presets: wording corrected to the shipped behaviour, the Image Trace button opens the controls; batch exporting, #1935; document window scroll bars, #2322; Anchor Point tool: a click on a handle end retracts that handle only, #2267; scores unchanged; Smart Guides: the dragged end of a straight segment keeps to its extension at any angle, #2120; Warp Options controls verified by focused checks and original-image review; estimates unchanged; existing-guide Escape cancellation validated 2026-10-14; scores and hours unchanged; open-path anchor Delete keeps surviving runs separate; estimates unchanged; Artboard tab-switch follow-up historically validated on `a0671777`; scores and hours unchanged; Artboard clean active-tab close/Delete follow-up validated 2026-10-15; Curvature curve geometry, #1236 in part; Width-point finite-arithmetic follow-up validated; scores and hours unchanged; Image Trace adjustment discovery; later 2026-10-14 merges: macOS save sheet Format menu back to missing, #1855, #866 reopened; anchor X/Y/W/H in the Transform panel, Properties and Control bar, #1824; Snap to Grid over Snap to Pixel, #1816; toolbar labels, columns and the All Tools drawer, #1751; Actions panel selection, #1629 in part; scores unchanged; 2026-10-14 merges: anchor Align and Distribute, #1636; Delete on closed paths and whole objects, #1674; selection inside isolation, #1666; Escape in a Direct Selection drag and while typing, #1743, #1686; Pen clicks under 3 px, #1663; [ and ] on more tools, #1661 in part; fields commit on click away, #1787; command search keys, #1757; a guide's position in the Transform panel is a convenience with no parity claim, #1659; scores unchanged; intersect Smart Guides for drawn points and dragged anchors, #1602 in part; a ruler guide's position in the Transform panel, #1597 in part; Symbol Options before New Symbol, #1560; tool flyout widths, #1616; Pathfinder Trim, Merge and Crop strokes, #1570; scores unchanged; G1/M11: Artboard deletion and history selection normalization; history regressions and full workspace CI pass; scores and hours unchanged; Width Point Edit: Shift-double-click enables adjoining adjustment; plain double-click keeps it off; scores and hours unchanged; per-corner radius fields with a link toggle, split from #1594; Join merges anchors lying on one another, #1533; correction: Average and Join with a corner or smooth choice is missing, not done; guide objects take the guide colour and style and Lock Guides, #1534; Keyboard Shortcuts lists commands outside the menus, #1539; round corners next to curves and the Smooth tool's reach, #1512; Make Compound Path on a group, #1537; dialog slider rails, #1483; template layers stay in Preview in Outline view, #1486; Direct Selection takes the selected object's anchor where anchors coincide, #1508; Tabs panel stop drags, #1458; Split Into Grid dialog, #1462; Transform Objects and Reflect's angle, #1448 and #1460; Pathfinder panel on a group, #1450; value fields: readable selection, #1465 in part, and a focused field follows its value, #1457; scores unchanged; Transform dialogs have a live Preview and step with the arrow keys, #1412; transform tools, dialogs and Move act on direct-selected anchors, #1408; the bounding box hides during a corner drag, #1229; scores unchanged; dialogs focus their first field on every open, #1367, and a first field that starts greyed out once it is enabled, #1395; scores unchanged; Rectangular Grid and Polar Grid click dialogs take the skew, frame, compound path and Fill Grid options, #1377 in part, score unchanged; Live Corners: a Direct Selection click on a widget selects its corner, #1239; custom ruler origin, #1212; Artboard tool clicks fit the topmost editable object in one undo step (#1227); scores unchanged; Pathfinder single-result source names and SVG IDs retained, #1293; DXF control-point SPLINE closure read, #1299 follow-up; Document Setup: Edit Artboards closes the modal after applying valid settings; pointer-click regressions; scores and hours unchanged; Artboards: Escape exits editing; an active drag cancels first; shortcut-routing regression; scores and hours unchanged; Simplify preview/counts and valid rewritten-path selection (#1272); parity remains partial; Texture-effect drag redraws optimized (#1257); idle Windows interaction still unverified; scores unchanged; Curvature point display improved (#1236, in part); scores unchanged; Direct Selection: no-op anchor Align preserves Undo, Redo and saved state (#1180 follow-up); Eyedropper cursor distinguished from precise crosshair (#1269); scores unchanged; Eyedropper can read visible locked and outside-isolation sources (#1251); scores unchanged; Direct Selection: independent segment picks and Delete (#1278); scores unchanged; Adding paint to an unpainted zero-width stroke restores the 1 pt default (#1253); scores unchanged; macOS save sheets offer a Format menu, #866; Attributes fill-rule controls edit the compound when a member is selected, #1197; a marquee needs to touch a path, not just lie inside its fill, #1103; earlier: Live Corners round corners between curved sides, #1144) · **Target:** Adobe Illustrator 2026 (30.x; bezier drag previews follow the pointer back into the window and to the release, #834; Guides: a path moved by a press on its stroke or beside its edge, and a straight segment Direct Selection drags, snap by the nearest point of the path, landing exactly on a ruler guide (#3701); scores and estimates unchanged; previously:; Artboard Options honors Scale Artwork with Artboard for preset, orientation and typed-size changes; both Fits keep artwork stationary; scores and estimates unchanged; Tangent and straight-segment perpendicular snapping use saved Smart Guides Geometric Guides preferences and Control bar quick access (#2537); no View submenu; G13: gradient Method (Illustrator 30.0): a Method menu in the Gradient panel, Classic or Perceptual (`paint.editGradient`/`paint.setFill` `method`, saved in the native file only when Perceptual, so older files open as Classic); Perceptual mixes a linear or radial gradient's colours in OKLab, on the canvas and in raster exports, and SVG and PDF exports write it as an image at the Document Raster Effects resolution, as the public documentation describes; Classic draws as before; a dithered Perceptual fill takes dither's noise; score and hours unchanged; G13: gradient Dither (Illustrator 30.0): a Dither box in the Gradient panel (`paint.editGradient`/`paint.setFill` `dither`, saved in the native file) draws a linear or radial gradient fill with seeded fine noise that hides banding, on the canvas and in raster exports; SVG and PDF exports write the fill as an image at the Document Raster Effects resolution, as the public documentation describes dither; fills only, as documented; score and hours unchanged; Recolor Artwork Edit chips scroll within the available screen height (#3416); small palettes retain their tab height; scores and estimates unchanged; Blend panel Type and Ramp control step easing with a separate color Shift (#2539); Blend Options retains spacing and orientation; polynomial curves are original approximations; Panels dock into any dock group or the icon column by dragging (UI.6), workspaces keep their layout changes (UI.7), dock groups collapse to their tabs by a double-click (UI.8); scores unchanged; Shape Builder Gap Detection shares Live Paint’s spatial gap finder and temporary merge bridges (#2526); Custom is capped at 72 pt in the existing options dialog; 3D: original Standard, Diffuse, Top Left and Right lighting presets; command access, effective-setting recognition and preview/history; scores and estimates unchanged; one selected clip group's Properties panel has Edit Clipping Path and Edit Contents at its top, G1; Distribute Objects and Distribute Spacing space from a key guide (#3165) with the Distribute Spacing value; Distribute spaces selected ruler guides with the objects, #1597 in part; the selection tools take a clipping path on its outline, over the art it clips; Polygon and Star exact-size placement remembers the chosen count for later drags and dialog openings; Cancel preserves the previous count; Live trace result bounds and selection highlights exclude the retained bitmap except in source-image views; a ruler guide can be the key object Align aligns to, and selected guides align to a key object, #2143; Pencil: Alt key toggles to Smooth Tool, #2001; status-readout sample widths cached by language, document unit and pixels per point; status messages and readouts use stable responsive allocations sized for localized labels and ordinary coordinates; cached canvas dimensions match the renderer limit before cache-key creation, view construction and worker submission; cached canvas reprojection preserves scale during resize, rotation and DPI changes; on-canvas Mirror Repeat axis, #2877; Background Export progress dialog with Cancel, #2564; 3D lighting: one to ten colored lights, smooth colored Inflate, legacy white-light appearance preserved; surface batching and restored seam minimum; scores and estimates unchanged; Blend spine Anchor Point conversions, selection/stacking priority and Shift regression coverage (#2544); scores and hours unchanged; Artboard Options' X and Y with a reference point, #1876; scores unchanged; library panels: one per library, dockable and tabbable, with Persistent, #2603; the Selection tool threads text from the out port, #2572 in part; scores unchanged; Group Selection picks and moves one path of a compound path, #2441 in part; the Move dialog opens on the last move and its last checkboxes; scores unchanged; transparency grid visibility saved in native documents, #1968; Live Corners radius up to the neighbouring cuts, #2332; Image Trace quick presets: wording corrected to the shipped behaviour, the Image Trace button opens the controls; batch exporting, #1935; document window scroll bars, #2322; Anchor Point tool: a click on a handle end retracts that handle only, #2267; scores unchanged; Smart Guides: the dragged end of a straight segment keeps to its extension at any angle, #2120; Warp Options controls verified by focused checks and original-image review; estimates unchanged; existing-guide Escape cancellation validated 2026-10-14; scores and hours unchanged; open-path anchor Delete keeps surviving runs separate; estimates unchanged; Artboard tab-switch follow-up historically validated on `a0671777`; scores and hours unchanged; Artboard clean active-tab close/Delete follow-up validated 2026-10-15; Curvature curve geometry, #1236 in part; Width-point finite-arithmetic follow-up validated; scores and hours unchanged; Image Trace adjustment discovery; later 2026-10-14 merges: macOS save sheet Format menu back to missing, #1855, #866 reopened; anchor X/Y/W/H in the Transform panel, Properties and Control bar, #1824; Snap to Grid over Snap to Pixel, #1816; toolbar labels, columns and the All Tools drawer, #1751; Actions panel selection, #1629 in part; scores unchanged; 2026-10-14 merges: anchor Align and Distribute, #1636; Delete on closed paths and whole objects, #1674; selection inside isolation, #1666; Escape in a Direct Selection drag and while typing, #1743, #1686; Pen clicks under 3 px, #1663; [ and ] on more tools, #1661 in part; fields commit on click away, #1787; command search keys, #1757; a guide's position in the Transform panel is a convenience with no parity claim, #1659; scores unchanged; intersect Smart Guides for drawn points and dragged anchors, #1602 in part; a ruler guide's position in the Transform panel, #1597 in part; Symbol Options before New Symbol, #1560; tool flyout widths, #1616; Pathfinder Trim, Merge and Crop strokes, #1570; scores unchanged; G1/M11: Artboard deletion and history selection normalization; history regressions and full workspace CI pass; scores and hours unchanged; Width Point Edit: Shift-double-click enables adjoining adjustment; plain double-click keeps it off; scores and hours unchanged; per-corner radius fields with a link toggle, split from #1594; Join merges anchors lying on one another, #1533; correction: Average and Join with a corner or smooth choice is missing, not done; guide objects take the guide colour and style and Lock Guides, #1534; Keyboard Shortcuts lists commands outside the menus, #1539; round corners next to curves and the Smooth tool's reach, #1512; Make Compound Path on a group, #1537; dialog slider rails, #1483; template layers stay in Preview in Outline view, #1486; Direct Selection takes the selected object's anchor where anchors coincide, #1508; Tabs panel stop drags, #1458; Split Into Grid dialog, #1462; Transform Objects and Reflect's angle, #1448 and #1460; Pathfinder panel on a group, #1450; value fields: readable selection, #1465 in part, and a focused field follows its value, #1457; scores unchanged; Transform dialogs have a live Preview and step with the arrow keys, #1412; transform tools, dialogs and Move act on direct-selected anchors, #1408; the bounding box hides during a corner drag, #1229; scores unchanged; dialogs focus their first field on every open, #1367, and a first field that starts greyed out once it is enabled, #1395; scores unchanged; Rectangular Grid and Polar Grid click dialogs take the skew, frame, compound path and Fill Grid options, #1377 in part, score unchanged; Live Corners: a Direct Selection click on a widget selects its corner, #1239; custom ruler origin, #1212; Artboard tool clicks fit the topmost editable object in one undo step (#1227); scores unchanged; Pathfinder single-result source names and SVG IDs retained, #1293; DXF control-point SPLINE closure read, #1299 follow-up; Document Setup: Edit Artboards closes the modal after applying valid settings; pointer-click regressions; scores and hours unchanged; Artboards: Escape exits editing; an active drag cancels first; shortcut-routing regression; scores and hours unchanged; Simplify preview/counts and valid rewritten-path selection (#1272); parity remains partial; Texture-effect drag redraws optimized (#1257); idle Windows interaction still unverified; scores unchanged; Curvature point display improved (#1236, in part); scores unchanged; Direct Selection: no-op anchor Align preserves Undo, Redo and saved state (#1180 follow-up); Eyedropper cursor distinguished from precise crosshair (#1269); scores unchanged; Eyedropper can read visible locked and outside-isolation sources (#1251); scores unchanged; Direct Selection: independent segment picks and Delete (#1278); scores unchanged; Adding paint to an unpainted zero-width stroke restores the 1 pt default (#1253); scores unchanged; macOS save sheets offer a Format menu, #866; Attributes fill-rule controls edit the compound when a member is selected, #1197; a marquee needs to touch a path, not just lie inside its fill, #1103; earlier: Live Corners round corners between curved sides, #1144; interactive Mirror Repeat axis; Background Export progress dialog with Cancel, #2564; scores unchanged; 3D lighting: one to ten colored lights, smooth colored Inflate, legacy white-light appearance preserved; surface batching and restored seam minimum; scores and estimates unchanged; Blend spine Anchor Point conversions, selection/stacking priority and Shift regression coverage (#2544); scores and hours unchanged; Artboard Options' X and Y with a reference point, #1876; scores unchanged; library panels: one per library, dockable and tabbable, with Persistent, #2603; the Selection tool threads text from the out port, #2572 in part; scores unchanged; Group Selection picks and moves one path of a compound path, #2441 in part; the Move dialog opens on the last move and its last checkboxes; scores unchanged; transparency grid visibility saved in native documents, #1968; Live Corners radius up to the neighbouring cuts, #2332; Image Trace quick presets: wording corrected to the shipped behaviour, the Image Trace button opens the controls; batch exporting, #1935; document window scroll bars, #2322; Anchor Point tool: a click on a handle end retracts that handle only, #2267; scores unchanged; Smart Guides: the dragged end of a straight segment keeps to its extension at any angle, #2120; Warp Options controls verified by focused checks and original-image review; estimates unchanged; existing-guide Escape cancellation validated 2026-10-14; scores and hours unchanged; open-path anchor Delete keeps surviving runs separate; estimates unchanged; Artboard tab-switch follow-up historically validated on `a0671777`; scores and hours unchanged; Artboard clean active-tab close/Delete follow-up validated 2026-10-15; Curvature curve geometry, #1236 in part; Width-point finite-arithmetic follow-up validated; scores and hours unchanged; Image Trace adjustment discovery; later 2026-10-14 merges: macOS save sheet Format menu back to missing, #1855, #866 reopened; anchor X/Y/W/H in the Transform panel, Properties and Control bar, #1824; Snap to Grid over Snap to Pixel, #1816; toolbar labels, columns and the All Tools drawer, #1751; Actions panel selection, #1629 in part; scores unchanged; 2026-10-14 merges: anchor Align and Distribute, #1636; Delete on closed paths and whole objects, #1674; selection inside isolation, #1666; Escape in a Direct Selection drag and while typing, #1743, #1686; Pen clicks under 3 px, #1663; [ and ] on more tools, #1661 in part; fields commit on click away, #1787; command search keys, #1757; a guide's position in the Transform panel is a convenience with no parity claim, #1659; scores unchanged; intersect Smart Guides for drawn points and dragged anchors, #1602 in part; a ruler guide's position in the Transform panel, #1597 in part; Symbol Options before New Symbol, #1560; tool flyout widths, #1616; Pathfinder Trim, Merge and Crop strokes, #1570; scores unchanged; G1/M11: Artboard deletion and history selection normalization; history regressions and full workspace CI pass; scores and hours unchanged; Width Point Edit: Shift-double-click enables adjoining adjustment; plain double-click keeps it off; scores and hours unchanged; per-corner radius fields with a link toggle, split from #1594; Join merges anchors lying on one another, #1533; correction: Average and Join with a corner or smooth choice is missing, not done; guide objects take the guide colour and style and Lock Guides, #1534; Keyboard Shortcuts lists commands outside the menus, #1539; round corners next to curves and the Smooth tool's reach, #1512; Make Compound Path on a group, #1537; dialog slider rails, #1483; template layers stay in Preview in Outline view, #1486; Direct Selection takes the selected object's anchor where anchors coincide, #1508; Tabs panel stop drags, #1458; Split Into Grid dialog, #1462; Transform Objects and Reflect's angle, #1448 and #1460; Pathfinder panel on a group, #1450; value fields: readable selection, #1465 in part, and a focused field follows its value, #1457; scores unchanged; Transform dialogs have a live Preview and step with the arrow keys, #1412; transform tools, dialogs and Move act on direct-selected anchors, #1408; the bounding box hides during a corner drag, #1229; scores unchanged; dialogs focus their first field on every open, #1367, and a first field that starts greyed out once it is enabled, #1395; scores unchanged; Rectangular Grid and Polar Grid click dialogs take the skew, frame, compound path and Fill Grid options, #1377 in part, score unchanged; Live Corners: a Direct Selection click on a widget selects its corner, #1239; custom ruler origin, #1212; Artboard tool clicks fit the topmost editable object in one undo step (#1227); scores unchanged; Pathfinder single-result source names and SVG IDs retained, #1293; DXF control-point SPLINE closure read, #1299 follow-up; Document Setup: Edit Artboards closes the modal after applying valid settings; pointer-click regressions; scores and hours unchanged; Artboards: Escape exits editing; an active drag cancels first; shortcut-routing regression; scores and hours unchanged; Simplify preview/counts and valid rewritten-path selection (#1272); parity remains partial; Texture-effect drag redraws optimized (#1257); idle Windows interaction still unverified; scores unchanged; Curvature point display improved (#1236, in part); scores unchanged; Direct Selection: no-op anchor Align preserves Undo, Redo and saved state (#1180 follow-up); Eyedropper cursor distinguished from precise crosshair (#1269); scores unchanged; Eyedropper can read visible locked and outside-isolation sources (#1251); scores unchanged; Direct Selection: independent segment picks and Delete (#1278); scores unchanged; Adding paint to an unpainted zero-width stroke restores the 1 pt default (#1253); scores unchanged; macOS save sheets offer a Format menu, #866; Attributes fill-rule controls edit the compound when a member is selected, #1197; a marquee needs to touch a path, not just lie inside its fill, #1103; earlier: Live Corners round corners between curved sides, #1144) · **Target:** Adobe Illustrator 2026 (30.x; Artboard Options: Show Center Mark and Show Cross Hairs (`artboard.setProps`/`artboard.new` `showCenterMark`, `showCrossHairs`) draw a center cross and lines from the middle of each side through the center on the canvas only, in one undo step; video safe areas, video rulers and the pixel aspect ratio remain missing; Tangent and straight-segment perpendicular snapping use saved Smart Guides Geometric Guides preferences and Control bar quick access (#2537); no View submenu; G13: gradient Method (Illustrator 30.0): a Method menu in the Gradient panel, Classic or Perceptual (`paint.editGradient`/`paint.setFill` `method`, saved in the native file only when Perceptual, so older files open as Classic); Perceptual mixes a linear or radial gradient's colours in OKLab, on the canvas and in raster exports, and SVG and PDF exports write it as an image at the Document Raster Effects resolution, as the public documentation describes; Classic draws as before; a dithered Perceptual fill takes dither's noise; score and hours unchanged; G13: gradient Dither (Illustrator 30.0): a Dither box in the Gradient panel (`paint.editGradient`/`paint.setFill` `dither`, saved in the native file) draws a linear or radial gradient fill with seeded fine noise that hides banding, on the canvas and in raster exports; SVG and PDF exports write the fill as an image at the Document Raster Effects resolution, as the public documentation describes dither; fills only, as documented; score and hours unchanged; Recolor Artwork Edit chips scroll within the available screen height (#3416); small palettes retain their tab height; scores and estimates unchanged; Blend panel Type and Ramp control step easing with a separate color Shift (#2539); Blend Options retains spacing and orientation; polynomial curves are original approximations; Panels dock into any dock group or the icon column by dragging (UI.6), workspaces keep their layout changes (UI.7), dock groups collapse to their tabs by a double-click (UI.8); scores unchanged; Shape Builder Gap Detection shares Live Paint’s spatial gap finder and temporary merge bridges (#2526); Custom is capped at 72 pt in the existing options dialog; 3D: original Standard, Diffuse, Top Left and Right lighting presets; command access, effective-setting recognition and preview/history; scores and estimates unchanged; one selected clip group's Properties panel has Edit Clipping Path and Edit Contents at its top, G1; Distribute Objects and Distribute Spacing space from a key guide (#3165) with the Distribute Spacing value; Distribute spaces selected ruler guides with the objects, #1597 in part; the selection tools take a clipping path on its outline, over the art it clips; Polygon and Star exact-size placement remembers the chosen count for later drags and dialog openings; Cancel preserves the previous count; Live trace result bounds and selection highlights exclude the retained bitmap except in source-image views; a ruler guide can be the key object Align aligns to, and selected guides align to a key object, #2143; Pencil: Alt key toggles to Smooth Tool, #2001; status-readout sample widths cached by language, document unit and pixels per point; status messages and readouts use stable responsive allocations sized for localized labels and ordinary coordinates; cached canvas dimensions match the renderer limit before cache-key creation, view construction and worker submission; cached canvas reprojection preserves scale during resize, rotation and DPI changes; on-canvas Mirror Repeat axis, #2877; Background Export progress dialog with Cancel, #2564; 3D lighting: one to ten colored lights, smooth colored Inflate, legacy white-light appearance preserved; surface batching and restored seam minimum; scores and estimates unchanged; Blend spine Anchor Point conversions, selection/stacking priority and Shift regression coverage (#2544); scores and hours unchanged; Artboard Options' X and Y with a reference point, #1876; scores unchanged; library panels: one per library, dockable and tabbable, with Persistent, #2603; the Selection tool threads text from the out port, #2572 in part; scores unchanged; Group Selection picks and moves one path of a compound path, #2441 in part; the Move dialog opens on the last move and its last checkboxes; scores unchanged; transparency grid visibility saved in native documents, #1968; Live Corners radius up to the neighbouring cuts, #2332; Image Trace quick presets: wording corrected to the shipped behaviour, the Image Trace button opens the controls; batch exporting, #1935; document window scroll bars, #2322; Anchor Point tool: a click on a handle end retracts that handle only, #2267; scores unchanged; Smart Guides: the dragged end of a straight segment keeps to its extension at any angle, #2120; Warp Options controls verified by focused checks and original-image review; estimates unchanged; existing-guide Escape cancellation validated 2026-10-14; scores and hours unchanged; open-path anchor Delete keeps surviving runs separate; estimates unchanged; Artboard tab-switch follow-up historically validated on `a0671777`; scores and hours unchanged; Artboard clean active-tab close/Delete follow-up validated 2026-10-15; Curvature curve geometry, #1236 in part; Width-point finite-arithmetic follow-up validated; scores and hours unchanged; Image Trace adjustment discovery; later 2026-10-14 merges: macOS save sheet Format menu back to missing, #1855, #866 reopened; anchor X/Y/W/H in the Transform panel, Properties and Control bar, #1824; Snap to Grid over Snap to Pixel, #1816; toolbar labels, columns and the All Tools drawer, #1751; Actions panel selection, #1629 in part; scores unchanged; 2026-10-14 merges: anchor Align and Distribute, #1636; Delete on closed paths and whole objects, #1674; selection inside isolation, #1666; Escape in a Direct Selection drag and while typing, #1743, #1686; Pen clicks under 3 px, #1663; [ and ] on more tools, #1661 in part; fields commit on click away, #1787; command search keys, #1757; a guide's position in the Transform panel is a convenience with no parity claim, #1659; scores unchanged; intersect Smart Guides for drawn points and dragged anchors, #1602 in part; a ruler guide's position in the Transform panel, #1597 in part; Symbol Options before New Symbol, #1560; tool flyout widths, #1616; Pathfinder Trim, Merge and Crop strokes, #1570; scores unchanged; G1/M11: Artboard deletion and history selection normalization; history regressions and full workspace CI pass; scores and hours unchanged; Width Point Edit: Shift-double-click enables adjoining adjustment; plain double-click keeps it off; scores and hours unchanged; per-corner radius fields with a link toggle, split from #1594; Join merges anchors lying on one another, #1533; correction: Average and Join with a corner or smooth choice is missing, not done; guide objects take the guide colour and style and Lock Guides, #1534; Keyboard Shortcuts lists commands outside the menus, #1539; round corners next to curves and the Smooth tool's reach, #1512; Make Compound Path on a group, #1537; dialog slider rails, #1483; template layers stay in Preview in Outline view, #1486; Direct Selection takes the selected object's anchor where anchors coincide, #1508; Tabs panel stop drags, #1458; Split Into Grid dialog, #1462; Transform Objects and Reflect's angle, #1448 and #1460; Pathfinder panel on a group, #1450; value fields: readable selection, #1465 in part, and a focused field follows its value, #1457; scores unchanged; Transform dialogs have a live Preview and step with the arrow keys, #1412; transform tools, dialogs and Move act on direct-selected anchors, #1408; the bounding box hides during a corner drag, #1229; scores unchanged; dialogs focus their first field on every open, #1367, and a first field that starts greyed out once it is enabled, #1395; scores unchanged; Rectangular Grid and Polar Grid click dialogs take the skew, frame, compound path and Fill Grid options, #1377 in part, score unchanged; Live Corners: a Direct Selection click on a widget selects its corner, #1239; custom ruler origin, #1212; Artboard tool clicks fit the topmost editable object in one undo step (#1227); scores unchanged; Pathfinder single-result source names and SVG IDs retained, #1293; DXF control-point SPLINE closure read, #1299 follow-up; Document Setup: Edit Artboards closes the modal after applying valid settings; pointer-click regressions; scores and hours unchanged; Artboards: Escape exits editing; an active drag cancels first; shortcut-routing regression; scores and hours unchanged; Simplify preview/counts and valid rewritten-path selection (#1272); parity remains partial; Texture-effect drag redraws optimized (#1257); idle Windows interaction still unverified; scores unchanged; Curvature point display improved (#1236, in part); scores unchanged; Direct Selection: no-op anchor Align preserves Undo, Redo and saved state (#1180 follow-up); Eyedropper cursor distinguished from precise crosshair (#1269); scores unchanged; Eyedropper can read visible locked and outside-isolation sources (#1251); scores unchanged; Direct Selection: independent segment picks and Delete (#1278); scores unchanged; Adding paint to an unpainted zero-width stroke restores the 1 pt default (#1253); scores unchanged; macOS save sheets offer a Format menu, #866; Attributes fill-rule controls edit the compound when a member is selected, #1197; a marquee needs to touch a path, not just lie inside its fill, #1103; earlier: Live Corners round corners between curved sides, #1144; interactive Mirror Repeat axis; Background Export progress dialog with Cancel, #2564; scores unchanged; 3D lighting: one to ten colored lights, smooth colored Inflate, legacy white-light appearance preserved; surface batching and restored seam minimum; scores and estimates unchanged; Blend spine Anchor Point conversions, selection/stacking priority and Shift regression coverage (#2544); scores and hours unchanged; Artboard Options' X and Y with a reference point, #1876; scores unchanged; library panels: one per library, dockable and tabbable, with Persistent, #2603; the Selection tool threads text from the out port, #2572 in part; scores unchanged; Group Selection picks and moves one path of a compound path, #2441 in part; the Move dialog opens on the last move and its last checkboxes; scores unchanged; transparency grid visibility saved in native documents, #1968; Live Corners radius up to the neighbouring cuts, #2332; Image Trace quick presets: wording corrected to the shipped behaviour, the Image Trace button opens the controls; batch exporting, #1935; document window scroll bars, #2322; Anchor Point tool: a click on a handle end retracts that handle only, #2267; scores unchanged; Smart Guides: the dragged end of a straight segment keeps to its extension at any angle, #2120; Warp Options controls verified by focused checks and original-image review; estimates unchanged; existing-guide Escape cancellation validated 2026-10-14; scores and hours unchanged; open-path anchor Delete keeps surviving runs separate; estimates unchanged; Artboard tab-switch follow-up historically validated on `a0671777`; scores and hours unchanged; Artboard clean active-tab close/Delete follow-up validated 2026-10-15; Curvature curve geometry, #1236 in part; Width-point finite-arithmetic follow-up validated; scores and hours unchanged; Image Trace adjustment discovery; later 2026-10-14 merges: macOS save sheet Format menu back to missing, #1855, #866 reopened; anchor X/Y/W/H in the Transform panel, Properties and Control bar, #1824; Snap to Grid over Snap to Pixel, #1816; toolbar labels, columns and the All Tools drawer, #1751; Actions panel selection, #1629 in part; scores unchanged; 2026-10-14 merges: anchor Align and Distribute, #1636; Delete on closed paths and whole objects, #1674; selection inside isolation, #1666; Escape in a Direct Selection drag and while typing, #1743, #1686; Pen clicks under 3 px, #1663; [ and ] on more tools, #1661 in part; fields commit on click away, #1787; command search keys, #1757; a guide's position in the Transform panel is a convenience with no parity claim, #1659; scores unchanged; intersect Smart Guides for drawn points and dragged anchors, #1602 in part; a ruler guide's position in the Transform panel, #1597 in part; Symbol Options before New Symbol, #1560; tool flyout widths, #1616; Pathfinder Trim, Merge and Crop strokes, #1570; scores unchanged; G1/M11: Artboard deletion and history selection normalization; history regressions and full workspace CI pass; scores and hours unchanged; Width Point Edit: Shift-double-click enables adjoining adjustment; plain double-click keeps it off; scores and hours unchanged; per-corner radius fields with a link toggle, split from #1594; Join merges anchors lying on one another, #1533; correction: Average and Join with a corner or smooth choice is missing, not done; guide objects take the guide colour and style and Lock Guides, #1534; Keyboard Shortcuts lists commands outside the menus, #1539; round corners next to curves and the Smooth tool's reach, #1512; Make Compound Path on a group, #1537; dialog slider rails, #1483; template layers stay in Preview in Outline view, #1486; Direct Selection takes the selected object's anchor where anchors coincide, #1508; Tabs panel stop drags, #1458; Split Into Grid dialog, #1462; Transform Objects and Reflect's angle, #1448 and #1460; Pathfinder panel on a group, #1450; value fields: readable selection, #1465 in part, and a focused field follows its value, #1457; scores unchanged; Transform dialogs have a live Preview and step with the arrow keys, #1412; transform tools, dialogs and Move act on direct-selected anchors, #1408; the bounding box hides during a corner drag, #1229; scores unchanged; dialogs focus their first field on every open, #1367, and a first field that starts greyed out once it is enabled, #1395; scores unchanged; Rectangular Grid and Polar Grid click dialogs take the skew, frame, compound path and Fill Grid options, #1377 in part, score unchanged; Live Corners: a Direct Selection click on a widget selects its corner, #1239; custom ruler origin, #1212; Artboard tool clicks fit the topmost editable object in one undo step (#1227); scores unchanged; Pathfinder single-result source names and SVG IDs retained, #1293; DXF control-point SPLINE closure read, #1299 follow-up; Document Setup: Edit Artboards closes the modal after applying valid settings; pointer-click regressions; scores and hours unchanged; Artboards: Escape exits editing; an active drag cancels first; shortcut-routing regression; scores and hours unchanged; Simplify preview/counts and valid rewritten-path selection (#1272); parity remains partial; Texture-effect drag redraws optimized (#1257); idle Windows interaction still unverified; scores unchanged; Curvature point display improved (#1236, in part); scores unchanged; Direct Selection: no-op anchor Align preserves Undo, Redo and saved state (#1180 follow-up); Eyedropper cursor distinguished from precise crosshair (#1269); scores unchanged; Eyedropper can read visible locked and outside-isolation sources (#1251); scores unchanged; Direct Selection: independent segment picks and Delete (#1278); scores unchanged; Adding paint to an unpainted zero-width stroke restores the 1 pt default (#1253); scores unchanged; macOS save sheets offer a Format menu, #866; Attributes fill-rule controls edit the compound when a member is selected, #1197; a marquee needs to touch a path, not just lie inside its fill, #1103; earlier: Live Corners round corners between curved sides, #1144) · **Target:** Adobe Illustrator 2026 (30.x; bezier drag previews follow the pointer back into the window and to the release, #834; Guides: a path moved by a press on its stroke or beside its edge, and a straight segment Direct Selection drags, snap by the nearest point of the path, landing exactly on a ruler guide (#3701); scores and estimates unchanged; previously:; Artboard Options honors Scale Artwork with Artboard for preset, orientation and typed-size changes; both Fits keep artwork stationary; scores and estimates unchanged; Tangent and straight-segment perpendicular snapping use saved Smart Guides Geometric Guides preferences and Control bar quick access (#2537); no View submenu; G13: gradient Method (Illustrator 30.0): a Method menu in the Gradient panel, Classic or Perceptual (`paint.editGradient`/`paint.setFill` `method`, saved in the native file only when Perceptual, so older files open as Classic); Perceptual mixes a linear or radial gradient's colours in OKLab, on the canvas and in raster exports, and SVG and PDF exports write it as an image at the Document Raster Effects resolution, as the public documentation describes; Classic draws as before; a dithered Perceptual fill takes dither's noise; score and hours unchanged; G13: gradient Dither (Illustrator 30.0): a Dither box in the Gradient panel (`paint.editGradient`/`paint.setFill` `dither`, saved in the native file) draws a linear or radial gradient fill with seeded fine noise that hides banding, on the canvas and in raster exports; SVG and PDF exports write the fill as an image at the Document Raster Effects resolution, as the public documentation describes dither; fills only, as documented; score and hours unchanged; Recolor Artwork Edit chips scroll within the available screen height (#3416); small palettes retain their tab height; scores and estimates unchanged; Blend panel Type and Ramp control step easing with a separate color Shift (#2539); Blend Options retains spacing and orientation; polynomial curves are original approximations; Panels dock into any dock group or the icon column by dragging (UI.6), workspaces keep their layout changes (UI.7), dock groups collapse to their tabs by a double-click (UI.8); scores unchanged; Shape Builder Gap Detection shares Live Paint’s spatial gap finder and temporary merge bridges (#2526); Custom is capped at 72 pt in the existing options dialog; 3D: original Standard, Diffuse, Top Left and Right lighting presets; command access, effective-setting recognition and preview/history; scores and estimates unchanged; one selected clip group's Properties panel has Edit Clipping Path and Edit Contents at its top, G1; Distribute Objects and Distribute Spacing space from a key guide (#3165) with the Distribute Spacing value; Distribute spaces selected ruler guides with the objects, #1597 in part; the selection tools take a clipping path on its outline, over the art it clips; Polygon and Star exact-size placement remembers the chosen count for later drags and dialog openings; Cancel preserves the previous count; Live trace result bounds and selection highlights exclude the retained bitmap except in source-image views; a ruler guide can be the key object Align aligns to, and selected guides align to a key object, #2143; Pencil: Alt key toggles to Smooth Tool, #2001; status-readout sample widths cached by language, document unit and pixels per point; status messages and readouts use stable responsive allocations sized for localized labels and ordinary coordinates; cached canvas dimensions match the renderer limit before cache-key creation, view construction and worker submission; cached canvas reprojection preserves scale during resize, rotation and DPI changes; on-canvas Mirror Repeat axis, #2877; Background Export progress dialog with Cancel, #2564; 3D lighting: one to ten colored lights, smooth colored Inflate, legacy white-light appearance preserved; surface batching and restored seam minimum; scores and estimates unchanged; Blend spine Anchor Point conversions, selection/stacking priority and Shift regression coverage (#2544); scores and hours unchanged; Artboard Options' X and Y with a reference point, #1876; scores unchanged; library panels: one per library, dockable and tabbable, with Persistent, #2603; the Selection tool threads text from the out port, #2572 in part; scores unchanged; Group Selection picks and moves one path of a compound path, #2441 in part; the Move dialog opens on the last move and its last checkboxes; scores unchanged; transparency grid visibility saved in native documents, #1968; Live Corners radius up to the neighbouring cuts, #2332; Image Trace quick presets: wording corrected to the shipped behaviour, the Image Trace button opens the controls; batch exporting, #1935; document window scroll bars, #2322; Anchor Point tool: a click on a handle end retracts that handle only, #2267; scores unchanged; Smart Guides: the dragged end of a straight segment keeps to its extension at any angle, #2120; Warp Options controls verified by focused checks and original-image review; estimates unchanged; existing-guide Escape cancellation validated 2026-10-14; scores and hours unchanged; open-path anchor Delete keeps surviving runs separate; estimates unchanged; Artboard tab-switch follow-up historically validated on `a0671777`; scores and hours unchanged; Artboard clean active-tab close/Delete follow-up validated 2026-10-15; Curvature curve geometry, #1236 in part; Width-point finite-arithmetic follow-up validated; scores and hours unchanged; Image Trace adjustment discovery; later 2026-10-14 merges: macOS save sheet Format menu back to missing, #1855, #866 reopened; anchor X/Y/W/H in the Transform panel, Properties and Control bar, #1824; Snap to Grid over Snap to Pixel, #1816; toolbar labels, columns and the All Tools drawer, #1751; Actions panel selection, #1629 in part; scores unchanged; 2026-10-14 merges: anchor Align and Distribute, #1636; Delete on closed paths and whole objects, #1674; selection inside isolation, #1666; Escape in a Direct Selection drag and while typing, #1743, #1686; Pen clicks under 3 px, #1663; [ and ] on more tools, #1661 in part; fields commit on click away, #1787; command search keys, #1757; a guide's position in the Transform panel is a convenience with no parity claim, #1659; scores unchanged; intersect Smart Guides for drawn points and dragged anchors, #1602 in part; a ruler guide's position in the Transform panel, #1597 in part; Symbol Options before New Symbol, #1560; tool flyout widths, #1616; Pathfinder Trim, Merge and Crop strokes, #1570; scores unchanged; G1/M11: Artboard deletion and history selection normalization; history regressions and full workspace CI pass; scores and hours unchanged; Width Point Edit: Shift-double-click enables adjoining adjustment; plain double-click keeps it off; scores and hours unchanged; per-corner radius fields with a link toggle, split from #1594; Join merges anchors lying on one another, #1533; correction: Average and Join with a corner or smooth choice is missing, not done; guide objects take the guide colour and style and Lock Guides, #1534; Keyboard Shortcuts lists commands outside the menus, #1539; round corners next to curves and the Smooth tool's reach, #1512; Make Compound Path on a group, #1537; dialog slider rails, #1483; template layers stay in Preview in Outline view, #1486; Direct Selection takes the selected object's anchor where anchors coincide, #1508; Tabs panel stop drags, #1458; Split Into Grid dialog, #1462; Transform Objects and Reflect's angle, #1448 and #1460; Pathfinder panel on a group, #1450; value fields: readable selection, #1465 in part, and a focused field follows its value, #1457; scores unchanged; Transform dialogs have a live Preview and step with the arrow keys, #1412; transform tools, dialogs and Move act on direct-selected anchors, #1408; the bounding box hides during a corner drag, #1229; scores unchanged; dialogs focus their first field on every open, #1367, and a first field that starts greyed out once it is enabled, #1395; scores unchanged; Rectangular Grid and Polar Grid click dialogs take the skew, frame, compound path and Fill Grid options, #1377 in part, score unchanged; Live Corners: a Direct Selection click on a widget selects its corner, #1239; custom ruler origin, #1212; Artboard tool clicks fit the topmost editable object in one undo step (#1227); scores unchanged; Pathfinder single-result source names and SVG IDs retained, #1293; DXF control-point SPLINE closure read, #1299 follow-up; Document Setup: Edit Artboards closes the modal after applying valid settings; pointer-click regressions; scores and hours unchanged; Artboards: Escape exits editing; an active drag cancels first; shortcut-routing regression; scores and hours unchanged; Simplify preview/counts and valid rewritten-path selection (#1272); parity remains partial; Texture-effect drag redraws optimized (#1257); idle Windows interaction still unverified; scores unchanged; Curvature point display improved (#1236, in part); scores unchanged; Direct Selection: no-op anchor Align preserves Undo, Redo and saved state (#1180 follow-up); Eyedropper cursor distinguished from precise crosshair (#1269); scores unchanged; Eyedropper can read visible locked and outside-isolation sources (#1251); scores unchanged; Direct Selection: independent segment picks and Delete (#1278); scores unchanged; Adding paint to an unpainted zero-width stroke restores the 1 pt default (#1253); scores unchanged; macOS save sheets offer a Format menu, #866; Attributes fill-rule controls edit the compound when a member is selected, #1197; a marquee needs to touch a path, not just lie inside its fill, #1103; earlier: Live Corners round corners between curved sides, #1144; interactive Mirror Repeat axis; Background Export progress dialog with Cancel, #2564; scores unchanged; 3D lighting: one to ten colored lights, smooth colored Inflate, legacy white-light appearance preserved; surface batching and restored seam minimum; scores and estimates unchanged; Blend spine Anchor Point conversions, selection/stacking priority and Shift regression coverage (#2544); scores and hours unchanged; Artboard Options' X and Y with a reference point, #1876; scores unchanged; library panels: one per library, dockable and tabbable, with Persistent, #2603; the Selection tool threads text from the out port, #2572 in part; scores unchanged; Group Selection picks and moves one path of a compound path, #2441 in part; the Move dialog opens on the last move and its last checkboxes; scores unchanged; transparency grid visibility saved in native documents, #1968; Live Corners radius up to the neighbouring cuts, #2332; Image Trace quick presets: wording corrected to the shipped behaviour, the Image Trace button opens the controls; batch exporting, #1935; document window scroll bars, #2322; Anchor Point tool: a click on a handle end retracts that handle only, #2267; scores unchanged; Smart Guides: the dragged end of a straight segment keeps to its extension at any angle, #2120; Warp Options controls verified by focused checks and original-image review; estimates unchanged; existing-guide Escape cancellation validated 2026-10-14; scores and hours unchanged; open-path anchor Delete keeps surviving runs separate; estimates unchanged; Artboard tab-switch follow-up historically validated on `a0671777`; scores and hours unchanged; Artboard clean active-tab close/Delete follow-up validated 2026-10-15; Curvature curve geometry, #1236 in part; Width-point finite-arithmetic follow-up validated; scores and hours unchanged; Image Trace adjustment discovery; later 2026-10-14 merges: macOS save sheet Format menu back to missing, #1855, #866 reopened; anchor X/Y/W/H in the Transform panel, Properties and Control bar, #1824; Snap to Grid over Snap to Pixel, #1816; toolbar labels, columns and the All Tools drawer, #1751; Actions panel selection, #1629 in part; scores unchanged; 2026-10-14 merges: anchor Align and Distribute, #1636; Delete on closed paths and whole objects, #1674; selection inside isolation, #1666; Escape in a Direct Selection drag and while typing, #1743, #1686; Pen clicks under 3 px, #1663; [ and ] on more tools, #1661 in part; fields commit on click away, #1787; command search keys, #1757; a guide's position in the Transform panel is a convenience with no parity claim, #1659; scores unchanged; intersect Smart Guides for drawn points and dragged anchors, #1602 in part; a ruler guide's position in the Transform panel, #1597 in part; Symbol Options before New Symbol, #1560; tool flyout widths, #1616; Pathfinder Trim, Merge and Crop strokes, #1570; scores unchanged; G1/M11: Artboard deletion and history selection normalization; history regressions and full workspace CI pass; scores and hours unchanged; Width Point Edit: Shift-double-click enables adjoining adjustment; plain double-click keeps it off; scores and hours unchanged; per-corner radius fields with a link toggle, split from #1594; Join merges anchors lying on one another, #1533; correction: Average and Join with a corner or smooth choice is missing, not done; guide objects take the guide colour and style and Lock Guides, #1534; Keyboard Shortcuts lists commands outside the menus, #1539; round corners next to curves and the Smooth tool's reach, #1512; Make Compound Path on a group, #1537; dialog slider rails, #1483; template layers stay in Preview in Outline view, #1486; Direct Selection takes the selected object's anchor where anchors coincide, #1508; Tabs panel stop drags, #1458; Split Into Grid dialog, #1462; Transform Objects and Reflect's angle, #1448 and #1460; Pathfinder panel on a group, #1450; value fields: readable selection, #1465 in part, and a focused field follows its value, #1457; scores unchanged; Transform dialogs have a live Preview and step with the arrow keys, #1412; transform tools, dialogs and Move act on direct-selected anchors, #1408; the bounding box hides during a corner drag, #1229; scores unchanged; dialogs focus their first field on every open, #1367, and a first field that starts greyed out once it is enabled, #1395; scores unchanged; Rectangular Grid and Polar Grid click dialogs take the skew, frame, compound path and Fill Grid options, #1377 in part, score unchanged; Live Corners: a Direct Selection click on a widget selects its corner, #1239; custom ruler origin, #1212; Artboard tool clicks fit the topmost editable object in one undo step (#1227); scores unchanged; Pathfinder single-result source names and SVG IDs retained, #1293; DXF control-point SPLINE closure read, #1299 follow-up; Document Setup: Edit Artboards closes the modal after applying valid settings; pointer-click regressions; scores and hours unchanged; Artboards: Escape exits editing; an active drag cancels first; shortcut-routing regression; scores and hours unchanged; Simplify preview/counts and valid rewritten-path selection (#1272); parity remains partial; Texture-effect drag redraws optimized (#1257); idle Windows interaction still unverified; scores unchanged; Curvature point display improved (#1236, in part); scores unchanged; Direct Selection: no-op anchor Align preserves Undo, Redo and saved state (#1180 follow-up); Eyedropper cursor distinguished from precise crosshair (#1269); scores unchanged; Eyedropper can read visible locked and outside-isolation sources (#1251); scores unchanged; Direct Selection: independent segment picks and Delete (#1278); scores unchanged; Adding paint to an unpainted zero-width stroke restores the 1 pt default (#1253); scores unchanged; macOS save sheets offer a Format menu, #866; Attributes fill-rule controls edit the compound when a member is selected, #1197; a marquee needs to touch a path, not just lie inside its fill, #1103; earlier: Live Corners round corners between curved sides, #1144) · **Target:** Adobe Illustrator 2026 (30.x; Simplify: Object › Path › Simplify opens its slider at the Auto-Simplify level for the selection, previewing that result, instead of at 1 pt; partial-path simplify remains missing; Tangent and straight-segment perpendicular snapping use saved Smart Guides Geometric Guides preferences and Control bar quick access (#2537); no View submenu; G13: gradient Method (Illustrator 30.0): a Method menu in the Gradient panel, Classic or Perceptual (`paint.editGradient`/`paint.setFill` `method`, saved in the native file only when Perceptual, so older files open as Classic); Perceptual mixes a linear or radial gradient's colours in OKLab, on the canvas and in raster exports, and SVG and PDF exports write it as an image at the Document Raster Effects resolution, as the public documentation describes; Classic draws as before; a dithered Perceptual fill takes dither's noise; score and hours unchanged; G13: gradient Dither (Illustrator 30.0): a Dither box in the Gradient panel (`paint.editGradient`/`paint.setFill` `dither`, saved in the native file) draws a linear or radial gradient fill with seeded fine noise that hides banding, on the canvas and in raster exports; SVG and PDF exports write the fill as an image at the Document Raster Effects resolution, as the public documentation describes dither; fills only, as documented; score and hours unchanged; Recolor Artwork Edit chips scroll within the available screen height (#3416); small palettes retain their tab height; scores and estimates unchanged; Blend panel Type and Ramp control step easing with a separate color Shift (#2539); Blend Options retains spacing and orientation; polynomial curves are original approximations; Panels dock into any dock group or the icon column by dragging (UI.6), workspaces keep their layout changes (UI.7), dock groups collapse to their tabs by a double-click (UI.8); scores unchanged; Shape Builder Gap Detection shares Live Paint’s spatial gap finder and temporary merge bridges (#2526); Custom is capped at 72 pt in the existing options dialog; 3D: original Standard, Diffuse, Top Left and Right lighting presets; command access, effective-setting recognition and preview/history; scores and estimates unchanged; one selected clip group's Properties panel has Edit Clipping Path and Edit Contents at its top, G1; Distribute Objects and Distribute Spacing space from a key guide (#3165) with the Distribute Spacing value; Distribute spaces selected ruler guides with the objects, #1597 in part; the selection tools take a clipping path on its outline, over the art it clips; Polygon and Star exact-size placement remembers the chosen count for later drags and dialog openings; Cancel preserves the previous count; Live trace result bounds and selection highlights exclude the retained bitmap except in source-image views; a ruler guide can be the key object Align aligns to, and selected guides align to a key object, #2143; Pencil: Alt key toggles to Smooth Tool, #2001; status-readout sample widths cached by language, document unit and pixels per point; status messages and readouts use stable responsive allocations sized for localized labels and ordinary coordinates; cached canvas dimensions match the renderer limit before cache-key creation, view construction and worker submission; cached canvas reprojection preserves scale during resize, rotation and DPI changes; on-canvas Mirror Repeat axis, #2877; Background Export progress dialog with Cancel, #2564; 3D lighting: one to ten colored lights, smooth colored Inflate, legacy white-light appearance preserved; surface batching and restored seam minimum; scores and estimates unchanged; Blend spine Anchor Point conversions, selection/stacking priority and Shift regression coverage (#2544); scores and hours unchanged; Artboard Options' X and Y with a reference point, #1876; scores unchanged; library panels: one per library, dockable and tabbable, with Persistent, #2603; the Selection tool threads text from the out port, #2572 in part; scores unchanged; Group Selection picks and moves one path of a compound path, #2441 in part; the Move dialog opens on the last move and its last checkboxes; scores unchanged; transparency grid visibility saved in native documents, #1968; Live Corners radius up to the neighbouring cuts, #2332; Image Trace quick presets: wording corrected to the shipped behaviour, the Image Trace button opens the controls; batch exporting, #1935; document window scroll bars, #2322; Anchor Point tool: a click on a handle end retracts that handle only, #2267; scores unchanged; Smart Guides: the dragged end of a straight segment keeps to its extension at any angle, #2120; Warp Options controls verified by focused checks and original-image review; estimates unchanged; existing-guide Escape cancellation validated 2026-10-14; scores and hours unchanged; open-path anchor Delete keeps surviving runs separate; estimates unchanged; Artboard tab-switch follow-up historically validated on `a0671777`; scores and hours unchanged; Artboard clean active-tab close/Delete follow-up validated 2026-10-15; Curvature curve geometry, #1236 in part; Width-point finite-arithmetic follow-up validated; scores and hours unchanged; Image Trace adjustment discovery; later 2026-10-14 merges: macOS save sheet Format menu back to missing, #1855, #866 reopened; anchor X/Y/W/H in the Transform panel, Properties and Control bar, #1824; Snap to Grid over Snap to Pixel, #1816; toolbar labels, columns and the All Tools drawer, #1751; Actions panel selection, #1629 in part; scores unchanged; 2026-10-14 merges: anchor Align and Distribute, #1636; Delete on closed paths and whole objects, #1674; selection inside isolation, #1666; Escape in a Direct Selection drag and while typing, #1743, #1686; Pen clicks under 3 px, #1663; [ and ] on more tools, #1661 in part; fields commit on click away, #1787; command search keys, #1757; a guide's position in the Transform panel is a convenience with no parity claim, #1659; scores unchanged; intersect Smart Guides for drawn points and dragged anchors, #1602 in part; a ruler guide's position in the Transform panel, #1597 in part; Symbol Options before New Symbol, #1560; tool flyout widths, #1616; Pathfinder Trim, Merge and Crop strokes, #1570; scores unchanged; G1/M11: Artboard deletion and history selection normalization; history regressions and full workspace CI pass; scores and hours unchanged; Width Point Edit: Shift-double-click enables adjoining adjustment; plain double-click keeps it off; scores and hours unchanged; per-corner radius fields with a link toggle, split from #1594; Join merges anchors lying on one another, #1533; correction: Average and Join with a corner or smooth choice is missing, not done; guide objects take the guide colour and style and Lock Guides, #1534; Keyboard Shortcuts lists commands outside the menus, #1539; round corners next to curves and the Smooth tool's reach, #1512; Make Compound Path on a group, #1537; dialog slider rails, #1483; template layers stay in Preview in Outline view, #1486; Direct Selection takes the selected object's anchor where anchors coincide, #1508; Tabs panel stop drags, #1458; Split Into Grid dialog, #1462; Transform Objects and Reflect's angle, #1448 and #1460; Pathfinder panel on a group, #1450; value fields: readable selection, #1465 in part, and a focused field follows its value, #1457; scores unchanged; Transform dialogs have a live Preview and step with the arrow keys, #1412; transform tools, dialogs and Move act on direct-selected anchors, #1408; the bounding box hides during a corner drag, #1229; scores unchanged; dialogs focus their first field on every open, #1367, and a first field that starts greyed out once it is enabled, #1395; scores unchanged; Rectangular Grid and Polar Grid click dialogs take the skew, frame, compound path and Fill Grid options, #1377 in part, score unchanged; Live Corners: a Direct Selection click on a widget selects its corner, #1239; custom ruler origin, #1212; Artboard tool clicks fit the topmost editable object in one undo step (#1227); scores unchanged; Pathfinder single-result source names and SVG IDs retained, #1293; DXF control-point SPLINE closure read, #1299 follow-up; Document Setup: Edit Artboards closes the modal after applying valid settings; pointer-click regressions; scores and hours unchanged; Artboards: Escape exits editing; an active drag cancels first; shortcut-routing regression; scores and hours unchanged; Simplify preview/counts and valid rewritten-path selection (#1272); parity remains partial; Texture-effect drag redraws optimized (#1257); idle Windows interaction still unverified; scores unchanged; Curvature point display improved (#1236, in part); scores unchanged; Direct Selection: no-op anchor Align preserves Undo, Redo and saved state (#1180 follow-up); Eyedropper cursor distinguished from precise crosshair (#1269); scores unchanged; Eyedropper can read visible locked and outside-isolation sources (#1251); scores unchanged; Direct Selection: independent segment picks and Delete (#1278); scores unchanged; Adding paint to an unpainted zero-width stroke restores the 1 pt default (#1253); scores unchanged; macOS save sheets offer a Format menu, #866; Attributes fill-rule controls edit the compound when a member is selected, #1197; a marquee needs to touch a path, not just lie inside its fill, #1103; earlier: Live Corners round corners between curved sides, #1144; interactive Mirror Repeat axis; Background Export progress dialog with Cancel, #2564; scores unchanged; 3D lighting: one to ten colored lights, smooth colored Inflate, legacy white-light appearance preserved; surface batching and restored seam minimum; scores and estimates unchanged; Blend spine Anchor Point conversions, selection/stacking priority and Shift regression coverage (#2544); scores and hours unchanged; Artboard Options' X and Y with a reference point, #1876; scores unchanged; library panels: one per library, dockable and tabbable, with Persistent, #2603; the Selection tool threads text from the out port, #2572 in part; scores unchanged; Group Selection picks and moves one path of a compound path, #2441 in part; the Move dialog opens on the last move and its last checkboxes; scores unchanged; transparency grid visibility saved in native documents, #1968; Live Corners radius up to the neighbouring cuts, #2332; Image Trace quick presets: wording corrected to the shipped behaviour, the Image Trace button opens the controls; batch exporting, #1935; document window scroll bars, #2322; Anchor Point tool: a click on a handle end retracts that handle only, #2267; scores unchanged; Smart Guides: the dragged end of a straight segment keeps to its extension at any angle, #2120; Warp Options controls verified by focused checks and original-image review; estimates unchanged; existing-guide Escape cancellation validated 2026-10-14; scores and hours unchanged; open-path anchor Delete keeps surviving runs separate; estimates unchanged; Artboard tab-switch follow-up historically validated on `a0671777`; scores and hours unchanged; Artboard clean active-tab close/Delete follow-up validated 2026-10-15; Curvature curve geometry, #1236 in part; Width-point finite-arithmetic follow-up validated; scores and hours unchanged; Image Trace adjustment discovery; later 2026-10-14 merges: macOS save sheet Format menu back to missing, #1855, #866 reopened; anchor X/Y/W/H in the Transform panel, Properties and Control bar, #1824; Snap to Grid over Snap to Pixel, #1816; toolbar labels, columns and the All Tools drawer, #1751; Actions panel selection, #1629 in part; scores unchanged; 2026-10-14 merges: anchor Align and Distribute, #1636; Delete on closed paths and whole objects, #1674; selection inside isolation, #1666; Escape in a Direct Selection drag and while typing, #1743, #1686; Pen clicks under 3 px, #1663; [ and ] on more tools, #1661 in part; fields commit on click away, #1787; command search keys, #1757; a guide's position in the Transform panel is a convenience with no parity claim, #1659; scores unchanged; intersect Smart Guides for drawn points and dragged anchors, #1602 in part; a ruler guide's position in the Transform panel, #1597 in part; Symbol Options before New Symbol, #1560; tool flyout widths, #1616; Pathfinder Trim, Merge and Crop strokes, #1570; scores unchanged; G1/M11: Artboard deletion and history selection normalization; history regressions and full workspace CI pass; scores and hours unchanged; Width Point Edit: Shift-double-click enables adjoining adjustment; plain double-click keeps it off; scores and hours unchanged; per-corner radius fields with a link toggle, split from #1594; Join merges anchors lying on one another, #1533; correction: Average and Join with a corner or smooth choice is missing, not done; guide objects take the guide colour and style and Lock Guides, #1534; Keyboard Shortcuts lists commands outside the menus, #1539; round corners next to curves and the Smooth tool's reach, #1512; Make Compound Path on a group, #1537; dialog slider rails, #1483; template layers stay in Preview in Outline view, #1486; Direct Selection takes the selected object's anchor where anchors coincide, #1508; Tabs panel stop drags, #1458; Split Into Grid dialog, #1462; Transform Objects and Reflect's angle, #1448 and #1460; Pathfinder panel on a group, #1450; value fields: readable selection, #1465 in part, and a focused field follows its value, #1457; scores unchanged; Transform dialogs have a live Preview and step with the arrow keys, #1412; transform tools, dialogs and Move act on direct-selected anchors, #1408; the bounding box hides during a corner drag, #1229; scores unchanged; dialogs focus their first field on every open, #1367, and a first field that starts greyed out once it is enabled, #1395; scores unchanged; Rectangular Grid and Polar Grid click dialogs take the skew, frame, compound path and Fill Grid options, #1377 in part, score unchanged; Live Corners: a Direct Selection click on a widget selects its corner, #1239; custom ruler origin, #1212; Artboard tool clicks fit the topmost editable object in one undo step (#1227); scores unchanged; Pathfinder single-result source names and SVG IDs retained, #1293; DXF control-point SPLINE closure read, #1299 follow-up; Document Setup: Edit Artboards closes the modal after applying valid settings; pointer-click regressions; scores and hours unchanged; Artboards: Escape exits editing; an active drag cancels first; shortcut-routing regression; scores and hours unchanged; Simplify preview/counts and valid rewritten-path selection (#1272); parity remains partial; Texture-effect drag redraws optimized (#1257); idle Windows interaction still unverified; scores unchanged; Curvature point display improved (#1236, in part); scores unchanged; Direct Selection: no-op anchor Align preserves Undo, Redo and saved state (#1180 follow-up); Eyedropper cursor distinguished from precise crosshair (#1269); scores unchanged; Eyedropper can read visible locked and outside-isolation sources (#1251); scores unchanged; Direct Selection: independent segment picks and Delete (#1278); scores unchanged; Adding paint to an unpainted zero-width stroke restores the 1 pt default (#1253); scores unchanged; macOS save sheets offer a Format menu, #866; Attributes fill-rule controls edit the compound when a member is selected, #1197; a marquee needs to touch a path, not just lie inside its fill, #1103; earlier: Live Corners round corners between curved sides, #1144) · **Target:** Adobe Illustrator 2026 (30.x; bezier drag previews follow the pointer back into the window and to the release, #834; Guides: a path moved by a press on its stroke or beside its edge, and a straight segment Direct Selection drags, snap by the nearest point of the path, landing exactly on a ruler guide (#3701); scores and estimates unchanged; previously:; Artboard Options honors Scale Artwork with Artboard for preset, orientation and typed-size changes; both Fits keep artwork stationary; scores and estimates unchanged; Tangent and straight-segment perpendicular snapping use saved Smart Guides Geometric Guides preferences and Control bar quick access (#2537); no View submenu; G13: gradient Method (Illustrator 30.0): a Method menu in the Gradient panel, Classic or Perceptual (`paint.editGradient`/`paint.setFill` `method`, saved in the native file only when Perceptual, so older files open as Classic); Perceptual mixes a linear or radial gradient's colours in OKLab, on the canvas and in raster exports, and SVG and PDF exports write it as an image at the Document Raster Effects resolution, as the public documentation describes; Classic draws as before; a dithered Perceptual fill takes dither's noise; score and hours unchanged; G13: gradient Dither (Illustrator 30.0): a Dither box in the Gradient panel (`paint.editGradient`/`paint.setFill` `dither`, saved in the native file) draws a linear or radial gradient fill with seeded fine noise that hides banding, on the canvas and in raster exports; SVG and PDF exports write the fill as an image at the Document Raster Effects resolution, as the public documentation describes dither; fills only, as documented; score and hours unchanged; Recolor Artwork Edit chips scroll within the available screen height (#3416); small palettes retain their tab height; scores and estimates unchanged; Blend panel Type and Ramp control step easing with a separate color Shift (#2539); Blend Options retains spacing and orientation; polynomial curves are original approximations; Panels dock into any dock group or the icon column by dragging (UI.6), workspaces keep their layout changes (UI.7), dock groups collapse to their tabs by a double-click (UI.8); scores unchanged; Shape Builder Gap Detection shares Live Paint’s spatial gap finder and temporary merge bridges (#2526); Custom is capped at 72 pt in the existing options dialog; 3D: original Standard, Diffuse, Top Left and Right lighting presets; command access, effective-setting recognition and preview/history; scores and estimates unchanged; one selected clip group's Properties panel has Edit Clipping Path and Edit Contents at its top, G1; Distribute Objects and Distribute Spacing space from a key guide (#3165) with the Distribute Spacing value; Distribute spaces selected ruler guides with the objects, #1597 in part; the selection tools take a clipping path on its outline, over the art it clips; Polygon and Star exact-size placement remembers the chosen count for later drags and dialog openings; Cancel preserves the previous count; Live trace result bounds and selection highlights exclude the retained bitmap except in source-image views; a ruler guide can be the key object Align aligns to, and selected guides align to a key object, #2143; Pencil: Alt key toggles to Smooth Tool, #2001; status-readout sample widths cached by language, document unit and pixels per point; status messages and readouts use stable responsive allocations sized for localized labels and ordinary coordinates; cached canvas dimensions match the renderer limit before cache-key creation, view construction and worker submission; cached canvas reprojection preserves scale during resize, rotation and DPI changes; on-canvas Mirror Repeat axis, #2877; Background Export progress dialog with Cancel, #2564; 3D lighting: one to ten colored lights, smooth colored Inflate, legacy white-light appearance preserved; surface batching and restored seam minimum; scores and estimates unchanged; Blend spine Anchor Point conversions, selection/stacking priority and Shift regression coverage (#2544); scores and hours unchanged; Artboard Options' X and Y with a reference point, #1876; scores unchanged; library panels: one per library, dockable and tabbable, with Persistent, #2603; the Selection tool threads text from the out port, #2572 in part; scores unchanged; Group Selection picks and moves one path of a compound path, #2441 in part; the Move dialog opens on the last move and its last checkboxes; scores unchanged; transparency grid visibility saved in native documents, #1968; Live Corners radius up to the neighbouring cuts, #2332; Image Trace quick presets: wording corrected to the shipped behaviour, the Image Trace button opens the controls; batch exporting, #1935; document window scroll bars, #2322; Anchor Point tool: a click on a handle end retracts that handle only, #2267; scores unchanged; Smart Guides: the dragged end of a straight segment keeps to its extension at any angle, #2120; Warp Options controls verified by focused checks and original-image review; estimates unchanged; existing-guide Escape cancellation validated 2026-10-14; scores and hours unchanged; open-path anchor Delete keeps surviving runs separate; estimates unchanged; Artboard tab-switch follow-up historically validated on `a0671777`; scores and hours unchanged; Artboard clean active-tab close/Delete follow-up validated 2026-10-15; Curvature curve geometry, #1236 in part; Width-point finite-arithmetic follow-up validated; scores and hours unchanged; Image Trace adjustment discovery; later 2026-10-14 merges: macOS save sheet Format menu back to missing, #1855, #866 reopened; anchor X/Y/W/H in the Transform panel, Properties and Control bar, #1824; Snap to Grid over Snap to Pixel, #1816; toolbar labels, columns and the All Tools drawer, #1751; Actions panel selection, #1629 in part; scores unchanged; 2026-10-14 merges: anchor Align and Distribute, #1636; Delete on closed paths and whole objects, #1674; selection inside isolation, #1666; Escape in a Direct Selection drag and while typing, #1743, #1686; Pen clicks under 3 px, #1663; [ and ] on more tools, #1661 in part; fields commit on click away, #1787; command search keys, #1757; a guide's position in the Transform panel is a convenience with no parity claim, #1659; scores unchanged; intersect Smart Guides for drawn points and dragged anchors, #1602 in part; a ruler guide's position in the Transform panel, #1597 in part; Symbol Options before New Symbol, #1560; tool flyout widths, #1616; Pathfinder Trim, Merge and Crop strokes, #1570; scores unchanged; G1/M11: Artboard deletion and history selection normalization; history regressions and full workspace CI pass; scores and hours unchanged; Width Point Edit: Shift-double-click enables adjoining adjustment; plain double-click keeps it off; scores and hours unchanged; per-corner radius fields with a link toggle, split from #1594; Join merges anchors lying on one another, #1533; correction: Average and Join with a corner or smooth choice is missing, not done; guide objects take the guide colour and style and Lock Guides, #1534; Keyboard Shortcuts lists commands outside the menus, #1539; round corners next to curves and the Smooth tool's reach, #1512; Make Compound Path on a group, #1537; dialog slider rails, #1483; template layers stay in Preview in Outline view, #1486; Direct Selection takes the selected object's anchor where anchors coincide, #1508; Tabs panel stop drags, #1458; Split Into Grid dialog, #1462; Transform Objects and Reflect's angle, #1448 and #1460; Pathfinder panel on a group, #1450; value fields: readable selection, #1465 in part, and a focused field follows its value, #1457; scores unchanged; Transform dialogs have a live Preview and step with the arrow keys, #1412; transform tools, dialogs and Move act on direct-selected anchors, #1408; the bounding box hides during a corner drag, #1229; scores unchanged; dialogs focus their first field on every open, #1367, and a first field that starts greyed out once it is enabled, #1395; scores unchanged; Rectangular Grid and Polar Grid click dialogs take the skew, frame, compound path and Fill Grid options, #1377 in part, score unchanged; Live Corners: a Direct Selection click on a widget selects its corner, #1239; custom ruler origin, #1212; Artboard tool clicks fit the topmost editable object in one undo step (#1227); scores unchanged; Pathfinder single-result source names and SVG IDs retained, #1293; DXF control-point SPLINE closure read, #1299 follow-up; Document Setup: Edit Artboards closes the modal after applying valid settings; pointer-click regressions; scores and hours unchanged; Artboards: Escape exits editing; an active drag cancels first; shortcut-routing regression; scores and hours unchanged; Simplify preview/counts and valid rewritten-path selection (#1272); parity remains partial; Texture-effect drag redraws optimized (#1257); idle Windows interaction still unverified; scores unchanged; Curvature point display improved (#1236, in part); scores unchanged; Direct Selection: no-op anchor Align preserves Undo, Redo and saved state (#1180 follow-up); Eyedropper cursor distinguished from precise crosshair (#1269); scores unchanged; Eyedropper can read visible locked and outside-isolation sources (#1251); scores unchanged; Direct Selection: independent segment picks and Delete (#1278); scores unchanged; Adding paint to an unpainted zero-width stroke restores the 1 pt default (#1253); scores unchanged; macOS save sheets offer a Format menu, #866; Attributes fill-rule controls edit the compound when a member is selected, #1197; a marquee needs to touch a path, not just lie inside its fill, #1103; earlier: Live Corners round corners between curved sides, #1144; interactive Mirror Repeat axis; Background Export progress dialog with Cancel, #2564; scores unchanged; 3D lighting: one to ten colored lights, smooth colored Inflate, legacy white-light appearance preserved; surface batching and restored seam minimum; scores and estimates unchanged; Blend spine Anchor Point conversions, selection/stacking priority and Shift regression coverage (#2544); scores and hours unchanged; Artboard Options' X and Y with a reference point, #1876; scores unchanged; library panels: one per library, dockable and tabbable, with Persistent, #2603; the Selection tool threads text from the out port, #2572 in part; scores unchanged; Group Selection picks and moves one path of a compound path, #2441 in part; the Move dialog opens on the last move and its last checkboxes; scores unchanged; transparency grid visibility saved in native documents, #1968; Live Corners radius up to the neighbouring cuts, #2332; Image Trace quick presets: wording corrected to the shipped behaviour, the Image Trace button opens the controls; batch exporting, #1935; document window scroll bars, #2322; Anchor Point tool: a click on a handle end retracts that handle only, #2267; scores unchanged; Smart Guides: the dragged end of a straight segment keeps to its extension at any angle, #2120; Warp Options controls verified by focused checks and original-image review; estimates unchanged; existing-guide Escape cancellation validated 2026-10-14; scores and hours unchanged; open-path anchor Delete keeps surviving runs separate; estimates unchanged; Artboard tab-switch follow-up historically validated on `a0671777`; scores and hours unchanged; Artboard clean active-tab close/Delete follow-up validated 2026-10-15; Curvature curve geometry, #1236 in part; Width-point finite-arithmetic follow-up validated; scores and hours unchanged; Image Trace adjustment discovery; later 2026-10-14 merges: macOS save sheet Format menu back to missing, #1855, #866 reopened; anchor X/Y/W/H in the Transform panel, Properties and Control bar, #1824; Snap to Grid over Snap to Pixel, #1816; toolbar labels, columns and the All Tools drawer, #1751; Actions panel selection, #1629 in part; scores unchanged; 2026-10-14 merges: anchor Align and Distribute, #1636; Delete on closed paths and whole objects, #1674; selection inside isolation, #1666; Escape in a Direct Selection drag and while typing, #1743, #1686; Pen clicks under 3 px, #1663; [ and ] on more tools, #1661 in part; fields commit on click away, #1787; command search keys, #1757; a guide's position in the Transform panel is a convenience with no parity claim, #1659; scores unchanged; intersect Smart Guides for drawn points and dragged anchors, #1602 in part; a ruler guide's position in the Transform panel, #1597 in part; Symbol Options before New Symbol, #1560; tool flyout widths, #1616; Pathfinder Trim, Merge and Crop strokes, #1570; scores unchanged; G1/M11: Artboard deletion and history selection normalization; history regressions and full workspace CI pass; scores and hours unchanged; Width Point Edit: Shift-double-click enables adjoining adjustment; plain double-click keeps it off; scores and hours unchanged; per-corner radius fields with a link toggle, split from #1594; Join merges anchors lying on one another, #1533; correction: Average and Join with a corner or smooth choice is missing, not done; guide objects take the guide colour and style and Lock Guides, #1534; Keyboard Shortcuts lists commands outside the menus, #1539; round corners next to curves and the Smooth tool's reach, #1512; Make Compound Path on a group, #1537; dialog slider rails, #1483; template layers stay in Preview in Outline view, #1486; Direct Selection takes the selected object's anchor where anchors coincide, #1508; Tabs panel stop drags, #1458; Split Into Grid dialog, #1462; Transform Objects and Reflect's angle, #1448 and #1460; Pathfinder panel on a group, #1450; value fields: readable selection, #1465 in part, and a focused field follows its value, #1457; scores unchanged; Transform dialogs have a live Preview and step with the arrow keys, #1412; transform tools, dialogs and Move act on direct-selected anchors, #1408; the bounding box hides during a corner drag, #1229; scores unchanged; dialogs focus their first field on every open, #1367, and a first field that starts greyed out once it is enabled, #1395; scores unchanged; Rectangular Grid and Polar Grid click dialogs take the skew, frame, compound path and Fill Grid options, #1377 in part, score unchanged; Live Corners: a Direct Selection click on a widget selects its corner, #1239; custom ruler origin, #1212; Artboard tool clicks fit the topmost editable object in one undo step (#1227); scores unchanged; Pathfinder single-result source names and SVG IDs retained, #1293; DXF control-point SPLINE closure read, #1299 follow-up; Document Setup: Edit Artboards closes the modal after applying valid settings; pointer-click regressions; scores and hours unchanged; Artboards: Escape exits editing; an active drag cancels first; shortcut-routing regression; scores and hours unchanged; Simplify preview/counts and valid rewritten-path selection (#1272); parity remains partial; Texture-effect drag redraws optimized (#1257); idle Windows interaction still unverified; scores unchanged; Curvature point display improved (#1236, in part); scores unchanged; Direct Selection: no-op anchor Align preserves Undo, Redo and saved state (#1180 follow-up); Eyedropper cursor distinguished from precise crosshair (#1269); scores unchanged; Eyedropper can read visible locked and outside-isolation sources (#1251); scores unchanged; Direct Selection: independent segment picks and Delete (#1278); scores unchanged; Adding paint to an unpainted zero-width stroke restores the 1 pt default (#1253); scores unchanged; macOS save sheets offer a Format menu, #866; Attributes fill-rule controls edit the compound when a member is selected, #1197; a marquee needs to touch a path, not just lie inside its fill, #1103; earlier: Live Corners round corners between curved sides, #1144) · **Target:** Adobe Illustrator 2026 (30.x; Preferences › General › Constrain Angle: arrow keys, Shift drags and new objects follow it, #394; bezier drag previews follow the pointer back into the window and to the release, #834; Guides: a path moved by a press on its stroke or beside its edge, and a straight segment Direct Selection drags, snap by the nearest point of the path, landing exactly on a ruler guide (#3701); scores and estimates unchanged; previously:; Artboard Options honors Scale Artwork with Artboard for preset, orientation and typed-size changes; both Fits keep artwork stationary; scores and estimates unchanged; Tangent and straight-segment perpendicular snapping use saved Smart Guides Geometric Guides preferences and Control bar quick access (#2537); no View submenu; G13: gradient Method (Illustrator 30.0): a Method menu in the Gradient panel, Classic or Perceptual (`paint.editGradient`/`paint.setFill` `method`, saved in the native file only when Perceptual, so older files open as Classic); Perceptual mixes a linear or radial gradient's colours in OKLab, on the canvas and in raster exports, and SVG and PDF exports write it as an image at the Document Raster Effects resolution, as the public documentation describes; Classic draws as before; a dithered Perceptual fill takes dither's noise; score and hours unchanged; G13: gradient Dither (Illustrator 30.0): a Dither box in the Gradient panel (`paint.editGradient`/`paint.setFill` `dither`, saved in the native file) draws a linear or radial gradient fill with seeded fine noise that hides banding, on the canvas and in raster exports; SVG and PDF exports write the fill as an image at the Document Raster Effects resolution, as the public documentation describes dither; fills only, as documented; score and hours unchanged; Recolor Artwork Edit chips scroll within the available screen height (#3416); small palettes retain their tab height; scores and estimates unchanged; Blend panel Type and Ramp control step easing with a separate color Shift (#2539); Blend Options retains spacing and orientation; polynomial curves are original approximations; Panels dock into any dock group or the icon column by dragging (UI.6), workspaces keep their layout changes (UI.7), dock groups collapse to their tabs by a double-click (UI.8); scores unchanged; Shape Builder Gap Detection shares Live Paint’s spatial gap finder and temporary merge bridges (#2526); Custom is capped at 72 pt in the existing options dialog; 3D: original Standard, Diffuse, Top Left and Right lighting presets; command access, effective-setting recognition and preview/history; scores and estimates unchanged; one selected clip group's Properties panel has Edit Clipping Path and Edit Contents at its top, G1; Distribute Objects and Distribute Spacing space from a key guide (#3165) with the Distribute Spacing value; Distribute spaces selected ruler guides with the objects, #1597 in part; the selection tools take a clipping path on its outline, over the art it clips; Polygon and Star exact-size placement remembers the chosen count for later drags and dialog openings; Cancel preserves the previous count; Live trace result bounds and selection highlights exclude the retained bitmap except in source-image views; a ruler guide can be the key object Align aligns to, and selected guides align to a key object, #2143; Pencil: Alt key toggles to Smooth Tool, #2001; status-readout sample widths cached by language, document unit and pixels per point; status messages and readouts use stable responsive allocations sized for localized labels and ordinary coordinates; cached canvas dimensions match the renderer limit before cache-key creation, view construction and worker submission; cached canvas reprojection preserves scale during resize, rotation and DPI changes; on-canvas Mirror Repeat axis, #2877; Background Export progress dialog with Cancel, #2564; 3D lighting: one to ten colored lights, smooth colored Inflate, legacy white-light appearance preserved; surface batching and restored seam minimum; scores and estimates unchanged; Blend spine Anchor Point conversions, selection/stacking priority and Shift regression coverage (#2544); scores and hours unchanged; Artboard Options' X and Y with a reference point, #1876; scores unchanged; library panels: one per library, dockable and tabbable, with Persistent, #2603; the Selection tool threads text from the out port, #2572 in part; scores unchanged; Group Selection picks and moves one path of a compound path, #2441 in part; the Move dialog opens on the last move and its last checkboxes; scores unchanged; transparency grid visibility saved in native documents, #1968; Live Corners radius up to the neighbouring cuts, #2332; Image Trace quick presets: wording corrected to the shipped behaviour, the Image Trace button opens the controls; batch exporting, #1935; document window scroll bars, #2322; Anchor Point tool: a click on a handle end retracts that handle only, #2267; scores unchanged; Smart Guides: the dragged end of a straight segment keeps to its extension at any angle, #2120; Warp Options controls verified by focused checks and original-image review; estimates unchanged; existing-guide Escape cancellation validated 2026-10-14; scores and hours unchanged; open-path anchor Delete keeps surviving runs separate; estimates unchanged; Artboard tab-switch follow-up historically validated on `a0671777`; scores and hours unchanged; Artboard clean active-tab close/Delete follow-up validated 2026-10-15; Curvature curve geometry, #1236 in part; Width-point finite-arithmetic follow-up validated; scores and hours unchanged; Image Trace adjustment discovery; later 2026-10-14 merges: macOS save sheet Format menu back to missing, #1855, #866 reopened; anchor X/Y/W/H in the Transform panel, Properties and Control bar, #1824; Snap to Grid over Snap to Pixel, #1816; toolbar labels, columns and the All Tools drawer, #1751; Actions panel selection, #1629 in part; scores unchanged; 2026-10-14 merges: anchor Align and Distribute, #1636; Delete on closed paths and whole objects, #1674; selection inside isolation, #1666; Escape in a Direct Selection drag and while typing, #1743, #1686; Pen clicks under 3 px, #1663; [ and ] on more tools, #1661 in part; fields commit on click away, #1787; command search keys, #1757; a guide's position in the Transform panel is a convenience with no parity claim, #1659; scores unchanged; intersect Smart Guides for drawn points and dragged anchors, #1602 in part; a ruler guide's position in the Transform panel, #1597 in part; Symbol Options before New Symbol, #1560; tool flyout widths, #1616; Pathfinder Trim, Merge and Crop strokes, #1570; scores unchanged; G1/M11: Artboard deletion and history selection normalization; history regressions and full workspace CI pass; scores and hours unchanged; Width Point Edit: Shift-double-click enables adjoining adjustment; plain double-click keeps it off; scores and hours unchanged; per-corner radius fields with a link toggle, split from #1594; Join merges anchors lying on one another, #1533; correction: Average and Join with a corner or smooth choice is missing, not done; guide objects take the guide colour and style and Lock Guides, #1534; Keyboard Shortcuts lists commands outside the menus, #1539; round corners next to curves and the Smooth tool's reach, #1512; Make Compound Path on a group, #1537; dialog slider rails, #1483; template layers stay in Preview in Outline view, #1486; Direct Selection takes the selected object's anchor where anchors coincide, #1508; Tabs panel stop drags, #1458; Split Into Grid dialog, #1462; Transform Objects and Reflect's angle, #1448 and #1460; Pathfinder panel on a group, #1450; value fields: readable selection, #1465 in part, and a focused field follows its value, #1457; scores unchanged; Transform dialogs have a live Preview and step with the arrow keys, #1412; transform tools, dialogs and Move act on direct-selected anchors, #1408; the bounding box hides during a corner drag, #1229; scores unchanged; dialogs focus their first field on every open, #1367, and a first field that starts greyed out once it is enabled, #1395; scores unchanged; Rectangular Grid and Polar Grid click dialogs take the skew, frame, compound path and Fill Grid options, #1377 in part, score unchanged; Live Corners: a Direct Selection click on a widget selects its corner, #1239; custom ruler origin, #1212; Artboard tool clicks fit the topmost editable object in one undo step (#1227); scores unchanged; Pathfinder single-result source names and SVG IDs retained, #1293; DXF control-point SPLINE closure read, #1299 follow-up; Document Setup: Edit Artboards closes the modal after applying valid settings; pointer-click regressions; scores and hours unchanged; Artboards: Escape exits editing; an active drag cancels first; shortcut-routing regression; scores and hours unchanged; Simplify preview/counts and valid rewritten-path selection (#1272); parity remains partial; Texture-effect drag redraws optimized (#1257); idle Windows interaction still unverified; scores unchanged; Curvature point display improved (#1236, in part); scores unchanged; Direct Selection: no-op anchor Align preserves Undo, Redo and saved state (#1180 follow-up); Eyedropper cursor distinguished from precise crosshair (#1269); scores unchanged; Eyedropper can read visible locked and outside-isolation sources (#1251); scores unchanged; Direct Selection: independent segment picks and Delete (#1278); scores unchanged; Adding paint to an unpainted zero-width stroke restores the 1 pt default (#1253); scores unchanged; macOS save sheets offer a Format menu, #866; Attributes fill-rule controls edit the compound when a member is selected, #1197; a marquee needs to touch a path, not just lie inside its fill, #1103; earlier: Live Corners round corners between curved sides, #1144; interactive Mirror Repeat axis; Background Export progress dialog with Cancel, #2564; scores unchanged; 3D lighting: one to ten colored lights, smooth colored Inflate, legacy white-light appearance preserved; surface batching and restored seam minimum; scores and estimates unchanged; Blend spine Anchor Point conversions, selection/stacking priority and Shift regression coverage (#2544); scores and hours unchanged; Artboard Options' X and Y with a reference point, #1876; scores unchanged; library panels: one per library, dockable and tabbable, with Persistent, #2603; the Selection tool threads text from the out port, #2572 in part; scores unchanged; Group Selection picks and moves one path of a compound path, #2441 in part; the Move dialog opens on the last move and its last checkboxes; scores unchanged; transparency grid visibility saved in native documents, #1968; Live Corners radius up to the neighbouring cuts, #2332; Image Trace quick presets: wording corrected to the shipped behaviour, the Image Trace button opens the controls; batch exporting, #1935; document window scroll bars, #2322; Anchor Point tool: a click on a handle end retracts that handle only, #2267; scores unchanged; Smart Guides: the dragged end of a straight segment keeps to its extension at any angle, #2120; Warp Options controls verified by focused checks and original-image review; estimates unchanged; existing-guide Escape cancellation validated 2026-10-14; scores and hours unchanged; open-path anchor Delete keeps surviving runs separate; estimates unchanged; Artboard tab-switch follow-up historically validated on `a0671777`; scores and hours unchanged; Artboard clean active-tab close/Delete follow-up validated 2026-10-15; Curvature curve geometry, #1236 in part; Width-point finite-arithmetic follow-up validated; scores and hours unchanged; Image Trace adjustment discovery; later 2026-10-14 merges: macOS save sheet Format menu back to missing, #1855, #866 reopened; anchor X/Y/W/H in the Transform panel, Properties and Control bar, #1824; Snap to Grid over Snap to Pixel, #1816; toolbar labels, columns and the All Tools drawer, #1751; Actions panel selection, #1629 in part; scores unchanged; 2026-10-14 merges: anchor Align and Distribute, #1636; Delete on closed paths and whole objects, #1674; selection inside isolation, #1666; Escape in a Direct Selection drag and while typing, #1743, #1686; Pen clicks under 3 px, #1663; [ and ] on more tools, #1661 in part; fields commit on click away, #1787; command search keys, #1757; a guide's position in the Transform panel is a convenience with no parity claim, #1659; scores unchanged; intersect Smart Guides for drawn points and dragged anchors, #1602 in part; a ruler guide's position in the Transform panel, #1597 in part; Symbol Options before New Symbol, #1560; tool flyout widths, #1616; Pathfinder Trim, Merge and Crop strokes, #1570; scores unchanged; G1/M11: Artboard deletion and history selection normalization; history regressions and full workspace CI pass; scores and hours unchanged; Width Point Edit: Shift-double-click enables adjoining adjustment; plain double-click keeps it off; scores and hours unchanged; per-corner radius fields with a link toggle, split from #1594; Join merges anchors lying on one another, #1533; correction: Average and Join with a corner or smooth choice is missing, not done; guide objects take the guide colour and style and Lock Guides, #1534; Keyboard Shortcuts lists commands outside the menus, #1539; round corners next to curves and the Smooth tool's reach, #1512; Make Compound Path on a group, #1537; dialog slider rails, #1483; template layers stay in Preview in Outline view, #1486; Direct Selection takes the selected object's anchor where anchors coincide, #1508; Tabs panel stop drags, #1458; Split Into Grid dialog, #1462; Transform Objects and Reflect's angle, #1448 and #1460; Pathfinder panel on a group, #1450; value fields: readable selection, #1465 in part, and a focused field follows its value, #1457; scores unchanged; Transform dialogs have a live Preview and step with the arrow keys, #1412; transform tools, dialogs and Move act on direct-selected anchors, #1408; the bounding box hides during a corner drag, #1229; scores unchanged; dialogs focus their first field on every open, #1367, and a first field that starts greyed out once it is enabled, #1395; scores unchanged; Rectangular Grid and Polar Grid click dialogs take the skew, frame, compound path and Fill Grid options, #1377 in part, score unchanged; Live Corners: a Direct Selection click on a widget selects its corner, #1239; custom ruler origin, #1212; Artboard tool clicks fit the topmost editable object in one undo step (#1227); scores unchanged; Pathfinder single-result source names and SVG IDs retained, #1293; DXF control-point SPLINE closure read, #1299 follow-up; Document Setup: Edit Artboards closes the modal after applying valid settings; pointer-click regressions; scores and hours unchanged; Artboards: Escape exits editing; an active drag cancels first; shortcut-routing regression; scores and hours unchanged; Simplify preview/counts and valid rewritten-path selection (#1272); parity remains partial; Texture-effect drag redraws optimized (#1257); idle Windows interaction still unverified; scores unchanged; Curvature point display improved (#1236, in part); scores unchanged; Direct Selection: no-op anchor Align preserves Undo, Redo and saved state (#1180 follow-up); Eyedropper cursor distinguished from precise crosshair (#1269); scores unchanged; Eyedropper can read visible locked and outside-isolation sources (#1251); scores unchanged; Direct Selection: independent segment picks and Delete (#1278); scores unchanged; Adding paint to an unpainted zero-width stroke restores the 1 pt default (#1253); scores unchanged; macOS save sheets offer a Format menu, #866; Attributes fill-rule controls edit the compound when a member is selected, #1197; a marquee needs to touch a path, not just lie inside its fill, #1103; earlier: Live Corners round corners between curved sides, #1144) · **Target:** Adobe Illustrator 2026 (30.x)

## Feature submission evidence
Sign in
Home
Shorts
Subscriptions
You
History
Sign in to like videos, comment, and subscribe.
Sign in
Explore
Music
Movies & TV
Live
Show more
More from YouTube

  
    
    
  
YouTube Premium

  
    
    
    
  
YouTube Music

  
    
      
      
    
    
      
      
    
  
YouTube Kids
Report history
AboutPressCopyrightContact usCreatorsAdvertiseDevelopersImpressumCancel Memberships
TermsPrivacyPolicy & SafetyHow YouTube worksTest new features
© 2026 Google LLC
All
Shorts
Unwatched
Watched
Videos
Recently uploaded
Live
About these results
Filters
Shorts
Step 1 for Monstera deliciosa: The BIG chop to propagate #monsteraplant #plantpropagation
1.6M views
Monstera deliciosa - Thai Constellation
30K views
This is how I grew my monstera deliciosa from seed! #timelapse #monstera
767K views
Never underestimate a well grown classic #monsteradeliciosa grown by @plantsdecoranddiy
299K views
The MOST DANGEROUS Fruit? - The Monstera Deliciosa Fruit!
229K views
true
11:21
Now playing
If I Only Knew These Monstera Tips 5 Years Ago
2M 3y ago
Sheffield Made Plants
My best Monstera care tips I wish I knew 5 years ago. -------------------- Download my FREE Plant Parent's Troubleshooting ...
9:12 ... actually two types of monstera plants you can buy that look very similar the monstera bozigiana and the monstera deliciosa they ...
4K
true
4:28
Now playing
Do this after you buy your first Monstera
204K 2y ago
Kill This Plant
Download My Houseplant App - https://blackthumbplant.app/ Join Our Discord for plant help - https://discord.gg/kw9FnABCRk ...
4K
Summary
Kill This Plant analyzes why a new Monstera may struggle by measuring light levels and soil moisture. Discover how to identify if a plant is receiving enough light and why adjusting the environment immediately after purchase is crucial for healthy growth.
true
8:12
Now playing
How to grow the BEST Monstera
1.3M 2y ago
Kill This Plant
Download My Houseplant App - https://blackthumbplant.app/ Join Our Discord for plant help - https://discord.gg/kw9FnABCRk ...
1 product
4K
Summary
Kill This Plant compares top cuttings versus rearranging existing vines to fix crawling Monsteras. Learn methods for staking, pruning for propagation, and managing soil requirements for healthier, upright growth.
true
4:06
Now playing
how to grow a monstera as fast as possible
57K 9mo ago
Kill This Plant
Download My Houseplant App - https://blackthumbplant.app/ Join Our Discord for plant help - https://discord.gg/kw9FnABCRk ...
4K
Summary
Kill This Plant explains how to optimize monstera growth by mastering water saturation, soil drainage mixtures, and light intensity. Learn to use tools for testing soil moisture and tips for extending daylight hours to ensure consistent, rapid plant development.
true
8:01
Now playing
7 MONSTERA Plant Care Tips You Need to KNOW : Monstera Deliciousa
340K 3y ago
MonstroFarm
I want to help you grow beautiful Monstera's, which is Super-Easy. Here are 7 Plant Care Tips on how I care and make sure that ...
8 chapters
Intro | LIGHTING | TEMPERATURE AND HUMIDITY | SOIL MIXTURE | FERTILIZER | PRUNING | PEST MANAGEMENT | WATERING
true
2:42
Now playing
239 Days Monstera Deliciosa Time-Lapse – From Seed to Stunning Plant! 🌱
83K 1y ago
Interesting as FCK
Ever wondered how long it really takes to grow a Monstera deliciosa from seed? Here's the full 239-day journey captured in one ...
4K
true
10:22
Now playing
0:19
10:22
Set Up Your Monstera For Success
263K 2y ago
Sheffield Made Plants
Download my FREE Plant Parent's Troubleshooting Handbook https://resources.sheffieldmadeplants.com/handbook ▸ 1:1 ...
4K
Matching chapter
0:19 My Monstera
true
3:13
Now playing
why people keep Killing their Monstera
1M 3y ago
Kill This Plant
Hopefully this video will get you to stop killing your Monstera Deliciosa. *Links in this description contain affiliate links.
0:00 Monstera deliciosa is supposed to be one of the easiest plants but you would not believe how many people are asking for help but ...
4K
true
5:18
Now playing
Why you need to cut your monstera
337K 2y ago
Kill This Plant
Download My Houseplant App - https://blackthumbplant.app/ Join Our Discord for plant help - https://discord.gg/kw9FnABCRk ...
4K
Summary
Kill This Plant demonstrates the process of air layering a large Monstera to safely propagate new roots before cutting. This technique allows for maintaining large foliage while preparing the plant for a fresh start in a new pot.
true
49:16
Now playing
I Wish I Knew These Monstera Tips 5 Years Ago
562K 2y ago
Sheffield Made Plants
All my best Monstera tips in 1 video! -------------------- Download my FREE Plant Parent's Troubleshooting Handbook ...
0:00 Perforations or holes in the Le or monstera deliciosa is one of the most highly Sal offer Commodities in the world sort of if you ...
4K
true
7:22
Now playing
Do These 3 Things For The Largest Monstera Deliciosa
70K 3y ago
Techplant
How to grow monstera deliciosa the right way for large leaves. These tips will help you grow a really nice monstera deliciosa!
4K
Summary
Techplant clarifies common misconceptions about pruning lower leaves and recommends single-stalk configurations for easier management. Discover the importance of proper orientation when securing plants to supports, ensuring the aerial roots are pressed against the pole for maximum leaf growth.
true
12:30
Now playing
The World’s Largest Monstera Deliciosa Plant
62K 3y ago
SleepyLizard
Get your Monstera Delciosa fruit and avocados at: https://www.guacfarm.com.
Summary
SleepyLizard showcases a massive Monstera Deliciosa plantation focused on fruit production rather than decoration. The video explores harvesting the sweet fruit and the agricultural challenges of integrating these plants within an avocado grove.
true
11:06
Now playing
10 Must Do Tricks For The BEST Monstera
714K 3y ago
Sheffield Made Plants
Head to https://squarespace.com/sheffield to save 10% off your first purchase of a website or domain using code SHEFFIELD.
4K
true
10:59
Now playing
How To Propagate Your Monstera The Right Way
347K 2y ago
Sheffield Made Plants
Monstera propagation the right way -------------------- Download my FREE Plant Parent's Troubleshooting Handbook ...
0:00 This is The Ultimate Guide to propagating and multiplying your monstera deliciosa or swiss cheese plant so you can have more of ...
4K
true
16:00
Now playing
Why You shouldn’t buy a Monstera Deliciosa !
15K 1y ago
The Millennial Plant Dad
The Monstera Deliciosa may be an iconic house plant staple but the fact of the matter is that it's not for everyone, home or space !
4K
7 chapters
Intro | Space | How it is sold | M poles | Climbing | Fertilizing | Outro
true
7:34
Now playing
Eating The Tastiest Fruit In The World! - How to Eat Monstera Deliciosa Safely | Mexican Breadfruit
657K 3y ago
Greenthumbs Garden
Monstera Delcicosa is a popular houseplant all over the world. What many do not know is that when grown outside in the right ...
CC
6 chapters
The Monstera Delcicosa Fruit | Monstera Delcicosa Fruit Nutritional Benefits | When To Harvest Monstera Delcicosa Fruit | When Is It Safe To Eat Monstera Delcicosa Fruit | What To Do With Unripe Monstera Delcicosa Fruit | Tasting The Monstera Delcicosa Fruit
true
11:05
Now playing
Grow Your Pothos 3x FASTER
768K 2y ago
Sheffield Made Plants
Download my FREE Plant Parent's Troubleshooting Handbook https://resources.sheffieldmadeplants.com/handbook ▸ 1:1 ...
4K
Summary
Sheffield Made Plants shares techniques for maximizing pothos growth through strategic potting, stem pinning, and propagation methods. Learn how proper lighting, soil selection, and understanding root health contribute to fuller, faster-growing vines.
true
12:48
Now playing
Popular Plant Products I’ll NEVER Buy Again (And What I Use Instead)
272K 1y ago
Sheffield Made Plants
Download my FREE Plant Parent's Troubleshooting Handbook https://resources.sheffieldmadeplants.com/handbook ...
4K
10 chapters
Intro | Rooting Hormone | Miracle Grow | Clear Pots | Decorative pots | Grow lights | Velcro plant eyes | Leaf shine | Core poles | Scissors
true
44:26
Now playing
Inside an Indoor Plant Oasis | Full Tour
325K 1y ago
Sheffield Made Plants
Download my FREE Plant Parent's Troubleshooting Handbook https://resources.sheffieldmadeplants.com/handbook ...
4K
Summary
Richard provides a comprehensive look into a living space transformed into a dense indoor jungle featuring over 200 diverse house plants. The tour showcases popular species alongside unique finds, sharing propagation tips and creative styling methods for various leaf types.
true
10:21
Now playing
Changing THIS Will Make Your Spider Plant THRIVE
914K 1y ago
Sheffield Made Plants
Download my FREE Plant Parent's Troubleshooting Handbook https://resources.sheffieldmadeplants.com/handbook ...
4K
6 chapters
Intro | Leaf Problems | Why Does This Work | Fertilizer | pH | Water Conditioner
true
12:26
Now playing
Growing A MATURE MONSTERA | How Long Does It Take???
26K 3y ago
Everything Plants
Soltech Solutions 15% discount code 'everythingplants' https://glnk.io/0lyv/9 VeryPlants use the code EverythingPlants10 for 10% ...
true
16:40
Now playing
Tour of House Plant in their Natural Environment (outdoors!)
57K 5y ago
Plant Vibrations with Devin Wallien
In this longer video I take a walking tour of the neighborhood El Poblado in Medellin, Colombia showing many of the most popular ...
14:08 there's really nothing like it and then let's finish with many of our favorite plant the beautiful monstera deliciosa this plant has ...
CC
Shorts
The MOST DANGEROUS Fruit? - The Monstera Deliciosa Fruit!
229K views
The best way to get your monstera leaves to grow is to attach them to something for support!
2.9M views
Trying Monstera Deliciosa fruit and growing seeds! #monsteraplant #growingseeds
315K views
Giant Monstera! #plants #monsteradeliciosa
18K views
The 9 Monstera Deliciosa fruits are getting bigger and will soon reach maturity 💚 #monsteradeliciosa
64K views
true
10:49
Now playing
You’re (Probably) Killing Your Monstera
374K 3y ago
Sheffield Made Plants
You're probably killing your Monstera if you make this common mistake. -------------------- Download my FREE Plant Parent's ...
4K
Summary
Sheffield Made Plants explores why pruning monsteras won't create a bushier appearance like other plants. Instead, learn how to properly propagate and stake these climbers to achieve a lush look without sacrificing mature foliage.
true
41:52
Now playing
How to continuously grow a large Monstera
146K 1y ago
Sydney Plant Guy
I am chopping my Monstera adansonii in half & re-extend both: one with Moss to grow indoors, one with aroid mix to grow ...
4K
Summary
Sydney Plant Guy demonstrates the 'chop and extend' process to manage a large Monstera adansonii in an indoor setting. The video covers techniques for maintaining plant stability using moss poles and garden stakes while preparing for future growth.
true
23:55
Now playing
How To Shop & Care For MONSTERA DELICIOSA - Plant Care Light, Repot, Soil, Water Houseplant Care 101
80K 2y ago
Plant Life with Ashley Anita
monstera #monsteradeliciosa #monsteraplant Visit https://bit.ly/3rRx8hV and use code ASHLEY at checkout to get 10% off any ...
Summary
Ashley Anita guides viewers through the entire process of acquiring a new Monstera Deliciosa, from assessing plant health and pest checks at the nursery to the final potting steps. The episode covers selecting the right pot size, identifying when to repot, and creating the ideal, well-draining soil mix for optimal growth.
true
2:29
Now playing
How To Grow ~ Monstera
5.6K 3y ago
Urban Abundance
Also known as the "Swiss cheese plant", monstera deliciosa is a tropical plant that is often used as a house plant because it is so ...
0:00 Hey what's up guys it's farmer Jack and today we're going to talk about monstera deliciosa. So this plant is also known as The ...
true
6:41
Now playing
Grow Monstera From Seed ( 0 - 8 Months )
89K 4y ago
Techplant
In this video I show you how to start monstera seeds and grow them into small plants! Growing Monstera deliciosa from seed is ...
4K
true
23:26
Now playing
Houseplant Care 101: Monstera Care Tips - Monstera Watering, Feeding, & Repotting
17K 4y ago
Plant Life with Ashley Anita
houseplants #monstera #plants Visit https://bit.ly/3rRx8hV and use code ASHLEY at checkout to get 10% off any potting mix.
0:44 ... when it comes to caring for your monstera deliciosa now on these three topics i've already done very very detailed episodes i've ...
true
10:01
Now playing
How To Propagate MONSTERA DELICIOSA 2 EASY Ways (DETAILED)
1.7M 6y ago
Techplant
HowTo #Propagate #MonsteraDeliciosa How To Propagate Monstrea Deliciosa Cuttings Easily! I tried using two different methods ...
4K
6 chapters
Intro | Cutting | Water Cutting | Spagna Moss Cutting | Mother Plant | Outro
true
8:13
Now playing
Easy IKEA Jungle Box (DIY Greenhouse)
1.7M 2y ago
SerpaDesign
I've made plenty of IKEA projects before, and we thought it was time for another. Well, actually, it was my wife's idea. A few months ...
4K
Summary
SerpaDesign transforms a standard IKEA cabinet into a self-sustaining greenhouse for lush tropical plants. The build involves custom glass sealing, a cork-panelled background, and specific drainage features to support thriving foliage like Alocasia and various bromeliads.
true
35:28
Now playing
All you need to know about Monstera Deliciosa
610K 7y ago
Swedish Plantguys
We hope you will enjoy this video! And thank you for watching ! Chapters: Purchase start of the video Planting 08:14 Placement ...
true
12:47
Now playing
#91 Top 6 Common Indoor Plants That Can Grow In Water | Grow Houseplants Without Soil
11M 5y ago
The Green Earth
My top 6 Indoor Plants that are growing in just water without soil. Growing plants in water, whether houseplants or an indoor herb ...
2 products
4K
12 chapters
Introduction | Cutting of Satin Pothos | Cutting of Golden Pothos | Cutting of Jade Plant | Cutting of ZZ Plant | Cutting of Rubber Plant | Cutting of Snake Plant | Day 01 of Water Propagation | Day 20 of Water Propagation | Day 34 of Water Propagation | Day 45 of Water Propagation | Growing Indoor Plants in LECA
true
10:47
Now playing
I Stopped Killing Jade Plants Once I Knew This
562K 1y ago
Sheffield Made Plants
Download my FREE Plant Parent's Troubleshooting Handbook https://resources.sheffieldmadeplants.com/handbook ...
4K
Summary
Sheffield Made Plants shares techniques for cultivating healthy jade plants, emphasizing the importance of ample sunlight and proper watering habits. Learn how to encourage a bushier structure through strategic pruning and explore methods for propagating new plants from cuttings.
true
1:41:19
Now playing
The ULTIMATE Guide To The 10 Most Popular Plants
258K 2y ago
Sheffield Made Plants
Download my FREE Plant Parent's Troubleshooting Handbook https://resources.sheffieldmadeplants.com/handbook ▸ Book a ...
4K
true
21:20
Now playing
Houseplants I would NEVER buy again! (and some that I would)
54K 1y ago
Kaylee Ellen
Some houseplants just aren't worth it… but a few totally are! Here are the ones I'd buy again. Anthurium Rare Plant Index: ...
Summary
Kaylee Ellen shares her personal experiences with specific plant varieties, explaining why some species prove to be too high-maintenance for a relaxing collection. The discussion covers growth patterns, light requirements, and the effort needed to keep these popular indoor plants looking their best over time.
true
16:09
Now playing
Making Thick Trunk Bonsai from Jade Plant | in 2 Years | Pruning | Repotting | Portulacaria Afra
3.6M 3y ago
What a Plant
Now You can Buy My Premium Bonsai Soil Mix (From Link Below) https://amzn.to/358xd84 Appreciate Your Support Please ...
13 chapters
Intro | Pruning | Preparing Bonsai Pot | Repotting | 2nd Pruning | 1st Result | Repotting into Bigger Pot | 3rd Pruning | Preparing Bonsai Pot | Repotting | Growth After Few Months | 4th Pruning | Final Result
true
17:04
Now playing
AFTER I CREATED THIS METHOD I NEVER LOST AN ORCHIDS AGAIN
2.1M 1y ago
Wonder Garden
Welcome to Wonder Garden! In today's video, discover the ultimate game-changing method that ensures you'll never lose an ...
3 products
4K
Shorts
😲 8FT TALL Monstera Deliciosa Repot 🌿 PART 1 #monstera #houseplants #indoorplants #repotting #plants
13K views
Monstera Deliciosa Aurea Variegated 🪴✨🌙 #plants #plantlover #indoorplants #monstera #houseplants
7.3K views
New
She’s thriving ✨😍 #monstera #monsteradeliciosa #giantmonstera #plants #houseplants
961 views
New
5 Poisonous Fruits You Should Never Eat Unripe! ⚠️Monstera Deliciosa.
7.7K views
New
1 year Monstera Bulbasaur TC update. #monstera #philodendron #plantcare #anthurium #plantcaretips
19K views
true
18:21
Now playing
18 Gorgeous Houseplants That Thrive Without Sunlight
839K 1y ago
Natural Health Remedies
18 Gorgeous Houseplants That Thrive Without Sunlight Don't have a sunny window? No problem! In this video, we're showing ...
Summary
Natural Health Remedies explores diverse options for dimly lit spaces, from vining plants to flowering varieties. Learn specific care tips for each plant, including watering frequency and soil preferences, to ensure longevity without direct sunlight.
true
25:13
Now playing
Just one slice of onion can make Anthurium explode with so many beautiful flowers
542K 1y ago
Wonder Garden
Welcome to Wonder Garden! In today's video, discover an astonishing trick: just one slice of onion can make your Anthurium ...
2 products
4K
true
3:38
Now playing
STOP doing this to your Monstera Deliciosa
349K 4y ago
Kill This Plant
I get the idea behind it. But this is not doing what you think its doing. We need to stop pruning our Monstera Deliciosa. Pruning this ...
4K
Summary
Kill This Plant explains how pruning affects hormone pathways and growth in Monsteras, unlike branching plants like jade. They reveal how cutting stems reverts plants to a juvenile state with smaller leaves and fenestrations.
true
14:42
Now playing
The only 4 Monstera Deliciosa tips you need to know
91K 5mo ago
Kill This Plant
Download My Houseplant App - https://blackthumbplant.app/ Join Our Discord for plant help - https://discord.gg/kw9FnABCRk ...
4K
5 chapters
Find the back of the plant | Measure your light | Get grow lights | Take top cuttings | Conclusion
true
11:01
Now playing
Tips + Tricks For Monstera Deliciosa | ALL YOU NEED TO KNOW 🌿 Indoor Plant Tips For Beginners
47K 5y ago
The Jungle Haven
Monstera deliciosa care tips - swiss cheese plant care - monstera houseplant tips and tricks - indoor plant tips for beginners ...
true
41:52
Now playing
How to continuously grow a large Monstera
146K 1y ago
Sydney Plant Guy
I am chopping my Monstera adansonii in half & re-extend both: one with Moss to grow indoors, one with aroid mix to grow ...
4K
Summary
Sydney Plant Guy demonstrates the 'chop and extend' process to manage a large Monstera adansonii in an indoor setting. The video covers techniques for maintaining plant stability using moss poles and garden stakes while preparing for future growth.
Shorts
5 years of monstera growth! #monstera #plants
696K views
Repotting my NEW indoor monstera plant🪴 #plants #monstera #indoorplants #houseplants #monsteraplant
315K views
New
Converting my M. deliciosa ‘brazil form’ from grown on wood to grown on a moss pole
26K views
Deliciosa VS Borsigiana Monstera | They’re VERY Different - Which One is Yours?
203K views
I have 600 houseplants and my Monstera Deliciosa my absolute FAVORITE plant in my collection 🤍🪴🥹
83K views
true
5:46
Now playing
How to Grow Monstera Deliciosa
6.5K 3mo ago
Tanner The Planter
Day 21/30 of teaching you how to grow your houseplants in a simple way. My reference guide to growing 150 of the most popular ...
Summary
Tanner The Planter provides a comprehensive guide on Monstera deliciosa care, focusing on proper soil composition, effective watering techniques, and optimal lighting conditions. Learn how to encourage healthy root systems and leaf maturity while managing growth through staking.
true
10:15
Now playing
cutting down my GIANT Monstera Deliciosa / Taking Cuttings / Repotting / Feed & Care Nicole's Garden
80K 2y ago
Nicole’s Garden
Please Like and Subscribe! Affiliate links Ponytail sunhat (CAN) https://amzn.to/3UKGeeG Hori Hori Knife (CAN) ...
true
17:59
Now playing
Proper care for Monstera deliciosa! These 7 care tips are a MUST-know 🪴 (Swiss cheese plant)
40K 9mo ago
crazyplantboy
Want your Monstera deliciosa to grow better and healthier? Need an overview of the most important care tips so you can ...
4K
true
2:42
Now playing
239 Days Monstera Deliciosa Time-Lapse – From Seed to Stunning Plant! 🌱
83K 1y ago
Interesting as FCK
Ever wondered how long it really takes to grow a Monstera deliciosa from seed? Here's the full 239-day journey captured in one ...
4K
true
2:52
Now playing
How to Repot Monstera Deliciosa. IT'S EASY!
149K 3y ago
Tanner The Planter
In this video we discuss and demonstrate the general methods of properly re-potting the Monstera Deliciosa. Find Your PERFECT ...
5 chapters
Intro | Preparing the soil | Checking the roots | Repot | Watering
true
19:41
Now playing
Monstera Deliciosa: From Jungle Survivor to Icon 🌿
19K 1y ago
Plantrums
Watch next: The Orchid's Dark Past — Obsession, Mystery, and Survival → https://www.youtube.com/watch?v=q7VpOcKEpAw ...
4 products
7 chapters
Intro | Monstera’s Ancient History | Monstera’s Early Documentation | Monstera’s Biological Secrets | Monstera’s Rise to Fame | Monstera’s Ecological Impact | Monstera’s True Value
Shorts
Eating Monstera Deliciosa Fruit, Finally!
471K views
What to do with your monstera aerial roots? #monsteraplant #plantcaretips #plantcare
981K views
New
Monstera deliciosa (exotic plant that produces fruit) #shorts  #foodforest #tropical  #gardening
1.1K views
A lot of growth already in a year and a half for big mama Monstera Deliciosa. #monsteraplant
433K views
Monstera Deliciosa Repot #monstera #monsteraplant #repotting
49K views
true
11:25
Now playing
How to start Monstera the right way
61K 4mo ago
Kill This Plant
Download My Houseplant App - https://blackthumbplant.app/ Join Our Discord for plant help - https://discord.gg/kw9FnABCRk ...
4K
Summary
Kill This Plant provides a comprehensive guide to maximizing growth for single-stem Monsteras. The discussion covers essential care routines, including light intensity monitoring, selecting appropriate soil amendments to improve drainage, proper staking techniques, and safe fertilizing schedules to help plants thrive.
true
5:37
Now playing
How to Add Support & Prune Back Overgrown Monstera Deliciosa Plants
242K 3y ago
My City Plants
How to prune Monstera Deliciosa (complete guide) ...
5 products
Earns commission
Summary
My City Plants demonstrates techniques for managing large foliage using bamboo stakes, raffia, and clean pruners. Learn how to secure sprawling stems for a narrower profile and selectively cut back growth to maintain a balanced shape.
true
7:13
Now playing
You don’t need to repot your Monstera
80K 2y ago
Kill This Plant
Join our discord community https://discord.gg/RYXzghkuhk * Get 15% off Soltech Grow Lights with promo code: KTP2025 ...
4K
Summary
Kill This Plant explains how to determine when a Monstera actually needs a larger container based on root volume and soil moisture. Learn to identify the right balance of root mass to soil, and discover alternative solutions for managing water retention and light requirements.
true
19:37
Now playing
Saving a Giant: I got an 11+ year old Monstera Deliciosa Large Form
210K 3y ago
Leafy_Luster
MONSTERA DELICIOSA large form exists & I finally found one to call my own. Join me on this Massive Monstera Makeover to ...
4K
true
12:31
Now playing
The Full Story of my 11+ year old Monstera Deliciosa Large Form
108K 2y ago
Leafy_Luster
It's one of my most requested videos to make. THE UPDATE on my rescue Monstera Deliciosa! My rescue mission was quite ...
4K
Shorts
Repotting 7ft+ monstera #shorts #gardenup #monstera #plants
409K views
Monstera Fruit: The Strange Tropical Fruit 🍃🟢
102K views
let's propagate monstera #monstera #propagate #plants #soil #water #roots #areal #cutting #plants
1M views
Grow a BIG Monstera✨ #houseplants #plantcare #plantcaretips #monsteradeliciosa
3.7K views
If you love your monstera, set it free #houseplant #monstera #plantgrowth
351K views
true
14:33
Now playing
MONSTERA PROPAGATION: How to Propagate Monstera deliciosa (2023)
153K 4y ago
Plant Vibrations with Devin Wallien
My favorite Houseplant tools here: https://amzn.to/3sYJ3ta Yesssss, finally I have created a video on how to successfully ...
CC
true
5:00
Now playing
5 quick tips Monstera Deliciosa
88K 5y ago
Swedish Plantguys
In this video we give you 5 quick tips for your Monstera Deliciosa also know as Swiss cheese plant. Enjoy! And thank you for ...
4K
7 chapters
Intro | Juvenile Monstera | Repot | Support | Prune | Humidity | Outro
true
14:29
Now playing
How to Repot a Monstera Deliciosa Plant | Staking | Aerial Roots | Care Guide
37K 3y ago
Ana Perez
Hello friends, here's how to repot a monstera deliciosa plant and how to care for a monstera plant as well! Here are all the ...
4K
CC
12 chapters
intro | When to repot a monstera deliciosa? | What to do when a monstera is really root bound? | What type of potting soil to use for a monstera deliciosa? | What type of pot to use for a monstera? | Are moss poles necessary for a monstera? | What to do with the aerial roots of a monstera | What type of light does a monstera need? | How often should you water a monstera? | How much water should you give a monstera plant? | Fertilizing a monstera plant | Pest Preventions
true
17:27
Now playing
Monstera Deliciosa Single Node Propagation (0-9 Months)
609K 4y ago
Techplant
propagating monstera nodes. In this video I will show you the process of making single node cuttings of monstera deliciosa and ...
1 product
4K
true
9:13
Now playing
5 Things Monsteras HATE That Might Surprise You
387K 3y ago
Sheffield Made Plants
Monsteras aren't particularly fussy plants but there are some key things they hate that might actually surprise you.
Summary
Sheffield Made Plants explains how common care habits like misting, incorrect potting, and allowing dust buildup can harm Monsteras. Learn about the importance of stable humidity, proper root support, and leaf hygiene for optimal plant health and leaf development.
true
11:47
Now playing
MONSTERA DELICIOSA Care | Swiss Cheese Plant Tips & Tricks
187K 7y ago
Good Growing
For bonus content, subscribe to my Patreon ›› https://www.patreon.com/goodgrowing Do you like being trendy? Want plants to ...
11 chapters
Intro | General Info | Water | Light | Temperature | Humidity | Fertiliser | Propagation | Toxicity | Growth Care | Outro
Shorts
Monstera Propagation (My Step by Step Process)
313K views
I CHOPPED UP my Monstera to get it to thrive #houseplants #plants #houseplant #monstera
804K views
Monstera Deliciosa 🌿 Complete Care, Propagation, Height & Fruit Facts!#youtubeshorts#shorts#reels
2.2K views
New
Propagation Loop♻️🌿 #planttips #plants #monstera #propagation #plantcare #diy
18K views
Monstera Deliciosa 2023 Season is Here
1.8M views
true
10:39
Now playing
MONSTERA DELICIOSA CARE + PROPAGATION | indoor swiss cheese plant care
174K 6y ago
Good & Planty
this MONSTER of a care guide is here monstera leaves going yellow? not getting those fenestrations? is your swiss cheese ...
CC
6 chapters
Intro | Water | Light | Repotting | Temperature Humidity | Propagation
true
10:25
Now playing
Monstera richtig pflegen – der komplette Guide | feey 🌱
129K 3y ago
feey
In diesem Video zeigen wir dir alles, was du zur richtigen Pflege deiner Monstera wissen musst: Dazu zählt der richtige Standort, ...
2:46 Das hier ist der Klassiker der Monsteras: die Monstera deliciosa. Diese findest du bei uns im Shop in ganz vielen verschiedenen ...
4K
CC
true
7:35
Now playing
Watch this before you separate your Monstera
38K 1y ago
Kill This Plant
Get 15% off Soltech Grow Lights with promo code: KTP2025 https://glnk.io/0lyv/13 * GET 10% off the best professional ...
4K
Summary
Kill This Plant explains how to properly assess indoor lighting and choose the correct pot size to prevent root rot after propagating. Using a light meter and moisture probe, they demonstrate the relationship between soil volume, oxygen levels, and plant stress.
true
35:40
Now playing
My Monstera Jungle Mint Started Reverting...So I Chopped It All Back 👀
16K 6mo ago
plantsbymelissa
My Monstera jungle mint started reverting, so it was time to cut it back and restart. In this video I'm chopping it down, resetting the ...
4K
8 chapters
Overview | Time to chop | Repotting the base | Chunky pon | Watering | Cuttings | Back in my plant room | Outro
true
15:38
Now playing
Every Type of Monstera Deliciosa Explained (And Which One to Buy)
85K 2y ago
Kaylee Ellen
M E R C H : https://kayleeellen.creator-spring.com/? Monstera Vareities: ...
Summary
Kaylee Ellen outlines the differences between small and large form Monstera deliciosa, focusing on internodal spacing, growth patterns, and leaf characteristics. Learn which variety best suits your available space and aesthetic preferences through a detailed comparison of popular green and variegated cultivars.
Shorts
The MOST DANGEROUS Fruit? - The Monstera Deliciosa Fruit!
229K views
MONSTERA Propagation with 100% SUCCESS | Watch Me Turn a sad 😢 Plant into 11 Happy 🤓babies!
607K views
Monstera Deliciosa : The Delicious Monster Houseplant ! 🪴🍍#garden  #farming #organic
3.1M views
Monstera deliciosa fruit #asmr
901K views
Would you do this?! Full Monstera Cut Back!
456K views
true
9:13
Now playing
5 Things Monsteras HATE That Might Surprise You
387K 3y ago
Sheffield Made Plants
Monsteras aren't particularly fussy plants but there are some key things they hate that might actually surprise you.
Summary
Sheffield Made Plants explains how common care habits like misting, incorrect potting, and allowing dust buildup can harm Monsteras. Learn about the importance of stable humidity, proper root support, and leaf hygiene for optimal plant health and leaf development.
true
11:00
Now playing
How to water propagate a Monstera Deliciosa
64K 1y ago
Kill This Plant
Download My Houseplant App - https://blackthumbplant.app/ Join Our Discord for plant help - https://discord.gg/kw9FnABCRk ...
4K
Summary
Kill This Plant demonstrates how to take top and mid cuttings for propagation while ensuring healthy root development and foliage maintenance. Learn techniques to prevent rot, choose appropriate pot sizes based on root mass, and mix suitable soil for growth.
Shorts
Monstera deliciosa harvest!
4.4M views
Here’s 4 tips to grow a HUGE Monstera Deliciosa! 🌱🪴
286K views
The Secret, Edible Fruit Hiding on Your Houseplant #monstera #animation
713K views
Monstera deliciosa propagation in water❤️#shorts #youtubeshorts #like #viral #fyp
425K views
Monstera Deliciosa Fruit #shorts
51K views
true
2:41
Now playing
My Mutant Monstera Deliciosa Update
7.9K 2y ago
Techplant
Amazon Affiliate link to light https://amzn.to/3S7wJVN These are amazon affiliate links, they don't change the price, however after ...
4K
true
0:47
Now playing
Monstera Deliciosa Time Lapse (HUGE LEAF)
28K 3y ago
Interesting as FCK
In this plant time lapse, we'll be taking a look at a GIANT monstera deliciosa growing, one of my favorite plants in the world!
4K
true
Sign in
Home
Shorts
Subscriptions
You
History
Sign in to like videos, comment, and subscribe.
Sign in
Explore
Music
Movies & TV
Live
Show more
More from YouTube

  
    
    
  
YouTube Premium

  
    
    
    
  
YouTube Music

  
    
      
      
    
    
      
      
    
  
YouTube Kids
Report history
AboutPressCopyrightContact usCreatorsAdvertiseDevelopersImpressumCancel Memberships
TermsPrivacyPolicy & SafetyHow YouTube worksTest new features
© 2026 Google LLC
All
Shorts
Unwatched
Watched
Videos
Recently uploaded
Live
About these results
Filters
Shorts
Step 1 for Monstera deliciosa: The BIG chop to propagate #monsteraplant #plantpropagation
1.6M views
Monstera deliciosa - Thai Constellation
30K views
This is how I grew my monstera deliciosa from seed! #timelapse #monstera
767K views
Never underestimate a well grown classic #monsteradeliciosa grown by @plantsdecoranddiy
299K views
The MOST DANGEROUS Fruit? - The Monstera Deliciosa Fruit!
229K views
true
11:21
Now playing
If I Only Knew These Monstera Tips 5 Years Ago
2M 3y ago
Sheffield Made Plants
My best Monstera care tips I wish I knew 5 years ago. -------------------- Download my FREE Plant Parent's Troubleshooting ...
9:12 ... actually two types of monstera plants you can buy that look very similar the monstera bozigiana and the monstera deliciosa they ...
4K
true
4:28
Now playing
Do this after you buy your first Monstera
204K 2y ago
Kill This Plant
Download My Houseplant App - https://blackthumbplant.app/ Join Our Discord for plant help - https://discord.gg/kw9FnABCRk ...
4K
Summary
Kill This Plant analyzes why a new Monstera may struggle by measuring light levels and soil moisture. Discover how to identify if a plant is receiving enough light and why adjusting the environment immediately after purchase is crucial for healthy growth.
true
8:12
Now playing
How to grow the BEST Monstera
1.3M 2y ago
Kill This Plant
Download My Houseplant App - https://blackthumbplant.app/ Join Our Discord for plant help - https://discord.gg/kw9FnABCRk ...
1 product
4K
Summary
Kill This Plant compares top cuttings versus rearranging existing vines to fix crawling Monsteras. Learn methods for staking, pruning for propagation, and managing soil requirements for healthier, upright growth.
true
4:06
Now playing
how to grow a monstera as fast as possible
57K 9mo ago
Kill This Plant
Download My Houseplant App - https://blackthumbplant.app/ Join Our Discord for plant help - https://discord.gg/kw9FnABCRk ...
4K
Summary
Kill This Plant explains how to optimize monstera growth by mastering water saturation, soil drainage mixtures, and light intensity. Learn to use tools for testing soil moisture and tips for extending daylight hours to ensure consistent, rapid plant development.
true
8:01
Now playing
7 MONSTERA Plant Care Tips You Need to KNOW : Monstera Deliciousa
340K 3y ago
MonstroFarm
I want to help you grow beautiful Monstera's, which is Super-Easy. Here are 7 Plant Care Tips on how I care and make sure that ...
8 chapters
Intro | LIGHTING | TEMPERATURE AND HUMIDITY | SOIL MIXTURE | FERTILIZER | PRUNING | PEST MANAGEMENT | WATERING
true
2:42
Now playing
239 Days Monstera Deliciosa Time-Lapse – From Seed to Stunning Plant! 🌱
83K 1y ago
Interesting as FCK
Ever wondered how long it really takes to grow a Monstera deliciosa from seed? Here's the full 239-day journey captured in one ...
4K
true
10:22
Now playing
0:19
10:22
Set Up Your Monstera For Success
263K 2y ago
Sheffield Made Plants
Download my FREE Plant Parent's Troubleshooting Handbook https://resources.sheffieldmadeplants.com/handbook ▸ 1:1 ...
4K
Matching chapter
0:19 My Monstera
true
3:13
Now playing
why people keep Killing their Monstera
1M 3y ago
Kill This Plant
Hopefully this video will get you to stop killing your Monstera Deliciosa. *Links in this description contain affiliate links.
0:00 Monstera deliciosa is supposed to be one of the easiest plants but you would not believe how many people are asking for help but ...
4K
true
5:18
Now playing
Why you need to cut your monstera
337K 2y ago
Kill This Plant
Download My Houseplant App - https://blackthumbplant.app/ Join Our Discord for plant help - https://discord.gg/kw9FnABCRk ...
4K
Summary
Kill This Plant demonstrates the process of air layering a large Monstera to safely propagate new roots before cutting. This technique allows for maintaining large foliage while preparing the plant for a fresh start in a new pot.
true
49:16
Now playing
I Wish I Knew These Monstera Tips 5 Years Ago
562K 2y ago
Sheffield Made Plants
All my best Monstera tips in 1 video! -------------------- Download my FREE Plant Parent's Troubleshooting Handbook ...
0:00 Perforations or holes in the Le or monstera deliciosa is one of the most highly Sal offer Commodities in the world sort of if you ...
4K
true
7:22
Now playing
Do These 3 Things For The Largest Monstera Deliciosa
70K 3y ago
Techplant
How to grow monstera deliciosa the right way for large leaves. These tips will help you grow a really nice monstera deliciosa!
4K
Summary
Techplant clarifies common misconceptions about pruning lower leaves and recommends single-stalk configurations for easier management. Discover the importance of proper orientation when securing plants to supports, ensuring the aerial roots are pressed against the pole for maximum leaf growth.
true
12:30
Now playing
The World’s Largest Monstera Deliciosa Plant
62K 3y ago
SleepyLizard
Get your Monstera Delciosa fruit and avocados at: https://www.guacfarm.com.
Summary
SleepyLizard showcases a massive Monstera Deliciosa plantation focused on fruit production rather than decoration. The video explores harvesting the sweet fruit and the agricultural challenges of integrating these plants within an avocado grove.
true
11:06
Now playing
10 Must Do Tricks For The BEST Monstera
714K 3y ago
Sheffield Made Plants
Head to https://squarespace.com/sheffield to save 10% off your first purchase of a website or domain using code SHEFFIELD.
4K
true
10:59
Now playing
How To Propagate Your Monstera The Right Way
347K 2y ago
Sheffield Made Plants
Monstera propagation the right way -------------------- Download my FREE Plant Parent's Troubleshooting Handbook ...
0:00 This is The Ultimate Guide to propagating and multiplying your monstera deliciosa or swiss cheese plant so you can have more of ...
4K
true
16:00
Now playing
Why You shouldn’t buy a Monstera Deliciosa !
15K 1y ago
The Millennial Plant Dad
The Monstera Deliciosa may be an iconic house plant staple but the fact of the matter is that it's not for everyone, home or space !
4K
7 chapters
Intro | Space | How it is sold | M poles | Climbing | Fertilizing | Outro
true
7:34
Now playing
Eating The Tastiest Fruit In The World! - How to Eat Monstera Deliciosa Safely | Mexican Breadfruit
657K 3y ago
Greenthumbs Garden
Monstera Delcicosa is a popular houseplant all over the world. What many do not know is that when grown outside in the right ...
CC
6 chapters
The Monstera Delcicosa Fruit | Monstera Delcicosa Fruit Nutritional Benefits | When To Harvest Monstera Delcicosa Fruit | When Is It Safe To Eat Monstera Delcicosa Fruit | What To Do With Unripe Monstera Delcicosa Fruit | Tasting The Monstera Delcicosa Fruit
true
11:05
Now playing
Grow Your Pothos 3x FASTER
768K 2y ago
Sheffield Made Plants
Download my FREE Plant Parent's Troubleshooting Handbook https://resources.sheffieldmadeplants.com/handbook ▸ 1:1 ...
4K
Summary
Sheffield Made Plants shares techniques for maximizing pothos growth through strategic potting, stem pinning, and propagation methods. Learn how proper lighting, soil selection, and understanding root health contribute to fuller, faster-growing vines.
true
12:48
Now playing
Popular Plant Products I’ll NEVER Buy Again (And What I Use Instead)
272K 1y ago
Sheffield Made Plants
Download my FREE Plant Parent's Troubleshooting Handbook https://resources.sheffieldmadeplants.com/handbook ...
4K
10 chapters
Intro | Rooting Hormone | Miracle Grow | Clear Pots | Decorative pots | Grow lights | Velcro plant eyes | Leaf shine | Core poles | Scissors
true
44:26
Now playing
Inside an Indoor Plant Oasis | Full Tour
325K 1y ago
Sheffield Made Plants
Download my FREE Plant Parent's Troubleshooting Handbook https://resources.sheffieldmadeplants.com/handbook ...
4K
Summary
Richard provides a comprehensive look into a living space transformed into a dense indoor jungle featuring over 200 diverse house plants. The tour showcases popular species alongside unique finds, sharing propagation tips and creative styling methods for various leaf types.
true
10:21
Now playing
Changing THIS Will Make Your Spider Plant THRIVE
914K 1y ago
Sheffield Made Plants
Download my FREE Plant Parent's Troubleshooting Handbook https://resources.sheffieldmadeplants.com/handbook ...
4K
6 chapters
Intro | Leaf Problems | Why Does This Work | Fertilizer | pH | Water Conditioner
true
12:26
Now playing
Growing A MATURE MONSTERA | How Long Does It Take???
26K 3y ago
Everything Plants
Soltech Solutions 15% discount code 'everythingplants' https://glnk.io/0lyv/9 VeryPlants use the code EverythingPlants10 for 10% ...
true
16:40
Now playing
Tour of House Plant in their Natural Environment (outdoors!)
57K 5y ago
Plant Vibrations with Devin Wallien
In this longer video I take a walking tour of the neighborhood El Poblado in Medellin, Colombia showing many of the most popular ...
14:08 there's really nothing like it and then let's finish with many of our favorite plant the beautiful monstera deliciosa this plant has ...
CC
Shorts
The MOST DANGEROUS Fruit? - The Monstera Deliciosa Fruit!
229K views
The best way to get your monstera leaves to grow is to attach them to something for support!
2.9M views
Trying Monstera Deliciosa fruit and growing seeds! #monsteraplant #growingseeds
315K views
Giant Monstera! #plants #monsteradeliciosa
18K views
The 9 Monstera Deliciosa fruits are getting bigger and will soon reach maturity 💚 #monsteradeliciosa
64K views
true
10:49
Now playing
You’re (Probably) Killing Your Monstera
374K 3y ago
Sheffield Made Plants
You're probably killing your Monstera if you make this common mistake. -------------------- Download my FREE Plant Parent's ...
4K
Summary
Sheffield Made Plants explores why pruning monsteras won't create a bushier appearance like other plants. Instead, learn how to properly propagate and stake these climbers to achieve a lush look without sacrificing mature foliage.
true
41:52
Now playing
How to continuously grow a large Monstera
146K 1y ago
Sydney Plant Guy
I am chopping my Monstera adansonii in half & re-extend both: one with Moss to grow indoors, one with aroid mix to grow ...
4K
Summary
Sydney Plant Guy demonstrates the 'chop and extend' process to manage a large Monstera adansonii in an indoor setting. The video covers techniques for maintaining plant stability using moss poles and garden stakes while preparing for future growth.
true
23:55
Now playing
How To Shop & Care For MONSTERA DELICIOSA - Plant Care Light, Repot, Soil, Water Houseplant Care 101
80K 2y ago
Plant Life with Ashley Anita
monstera #monsteradeliciosa #monsteraplant Visit https://bit.ly/3rRx8hV and use code ASHLEY at checkout to get 10% off any ...
Summary
Ashley Anita guides viewers through the entire process of acquiring a new Monstera Deliciosa, from assessing plant health and pest checks at the nursery to the final potting steps. The episode covers selecting the right pot size, identifying when to repot, and creating the ideal, well-draining soil mix for optimal growth.
true
2:29
Now playing
How To Grow ~ Monstera
5.6K 3y ago
Urban Abundance
Also known as the "Swiss cheese plant", monstera deliciosa is a tropical plant that is often used as a house plant because it is so ...
0:00 Hey what's up guys it's farmer Jack and today we're going to talk about monstera deliciosa. So this plant is also known as The ...
true
6:41
Now playing
Grow Monstera From Seed ( 0 - 8 Months )
89K 4y ago
Techplant
In this video I show you how to start monstera seeds and grow them into small plants! Growing Monstera deliciosa from seed is ...
4K
true
23:26
Now playing
Houseplant Care 101: Monstera Care Tips - Monstera Watering, Feeding, & Repotting
17K 4y ago
Plant Life with Ashley Anita
houseplants #monstera #plants Visit https://bit.ly/3rRx8hV and use code ASHLEY at checkout to get 10% off any potting mix.
0:44 ... when it comes to caring for your monstera deliciosa now on these three topics i've already done very very detailed episodes i've ...
true
10:01
Now playing
How To Propagate MONSTERA DELICIOSA 2 EASY Ways (DETAILED)
1.7M 6y ago
Techplant
HowTo #Propagate #MonsteraDeliciosa How To Propagate Monstrea Deliciosa Cuttings Easily! I tried using two different methods ...
4K
6 chapters
Intro | Cutting | Water Cutting | Spagna Moss Cutting | Mother Plant | Outro
true
8:13
Now playing
Easy IKEA Jungle Box (DIY Greenhouse)
1.7M 2y ago
SerpaDesign
I've made plenty of IKEA projects before, and we thought it was time for another. Well, actually, it was my wife's idea. A few months ...
4K
Summary
SerpaDesign transforms a standard IKEA cabinet into a self-sustaining greenhouse for lush tropical plants. The build involves custom glass sealing, a cork-panelled background, and specific drainage features to support thriving foliage like Alocasia and various bromeliads.
true
35:28
Now playing
All you need to know about Monstera Deliciosa
610K 7y ago
Swedish Plantguys
We hope you will enjoy this video! And thank you for watching ! Chapters: Purchase start of the video Planting 08:14 Placement ...
true
12:47
Now playing
#91 Top 6 Common Indoor Plants That Can Grow In Water | Grow Houseplants Without Soil
11M 5y ago
The Green Earth
My top 6 Indoor Plants that are growing in just water without soil. Growing plants in water, whether houseplants or an indoor herb ...
2 products
4K
12 chapters
Introduction | Cutting of Satin Pothos | Cutting of Golden Pothos | Cutting of Jade Plant | Cutting of ZZ Plant | Cutting of Rubber Plant | Cutting of Snake Plant | Day 01 of Water Propagation | Day 20 of Water Propagation | Day 34 of Water Propagation | Day 45 of Water Propagation | Growing Indoor Plants in LECA
true
10:47
Now playing
I Stopped Killing Jade Plants Once I Knew This
562K 1y ago
Sheffield Made Plants
Download my FREE Plant Parent's Troubleshooting Handbook https://resources.sheffieldmadeplants.com/handbook ...
4K
Summary
Sheffield Made Plants shares techniques for cultivating healthy jade plants, emphasizing the importance of ample sunlight and proper watering habits. Learn how to encourage a bushier structure through strategic pruning and explore methods for propagating new plants from cuttings.
true
1:41:19
Now playing
The ULTIMATE Guide To The 10 Most Popular Plants
258K 2y ago
Sheffield Made Plants
Download my FREE Plant Parent's Troubleshooting Handbook https://resources.sheffieldmadeplants.com/handbook ▸ Book a ...
4K
true
21:20
Now playing
Houseplants I would NEVER buy again! (and some that I would)
54K 1y ago
Kaylee Ellen
Some houseplants just aren't worth it… but a few totally are! Here are the ones I'd buy again. Anthurium Rare Plant Index: ...
Summary
Kaylee Ellen shares her personal experiences with specific plant varieties, explaining why some species prove to be too high-maintenance for a relaxing collection. The discussion covers growth patterns, light requirements, and the effort needed to keep these popular indoor plants looking their best over time.
true
16:09
Now playing
Making Thick Trunk Bonsai from Jade Plant | in 2 Years | Pruning | Repotting | Portulacaria Afra
3.6M 3y ago
What a Plant
Now You can Buy My Premium Bonsai Soil Mix (From Link Below) https://amzn.to/358xd84 Appreciate Your Support Please ...
13 chapters
Intro | Pruning | Preparing Bonsai Pot | Repotting | 2nd Pruning | 1st Result | Repotting into Bigger Pot | 3rd Pruning | Preparing Bonsai Pot | Repotting | Growth After Few Months | 4th Pruning | Final Result
true
17:04
Now playing
AFTER I CREATED THIS METHOD I NEVER LOST AN ORCHIDS AGAIN
2.1M 1y ago
Wonder Garden
Welcome to Wonder Garden! In today's video, discover the ultimate game-changing method that ensures you'll never lose an ...
3 products
4K
Shorts
😲 8FT TALL Monstera Deliciosa Repot 🌿 PART 1 #monstera #houseplants #indoorplants #repotting #plants
13K views
Monstera Deliciosa Aurea Variegated 🪴✨🌙 #plants #plantlover #indoorplants #monstera #houseplants
7.3K views
New
She’s thriving ✨😍 #monstera #monsteradeliciosa #giantmonstera #plants #houseplants
961 views
New
5 Poisonous Fruits You Should Never Eat Unripe! ⚠️Monstera Deliciosa.
7.7K views
New
1 year Monstera Bulbasaur TC update. #monstera #philodendron #plantcare #anthurium #plantcaretips
19K views
true
18:21
Now playing
18 Gorgeous Houseplants That Thrive Without Sunlight
839K 1y ago
Natural Health Remedies
18 Gorgeous Houseplants That Thrive Without Sunlight Don't have a sunny window? No problem! In this video, we're showing ...
Summary
Natural Health Remedies explores diverse options for dimly lit spaces, from vining plants to flowering varieties. Learn specific care tips for each plant, including watering frequency and soil preferences, to ensure longevity without direct sunlight.
true
25:13
Now playing
Just one slice of onion can make Anthurium explode with so many beautiful flowers
542K 1y ago
Wonder Garden
Welcome to Wonder Garden! In today's video, discover an astonishing trick: just one slice of onion can make your Anthurium ...
2 products
4K
true
3:38
Now playing
STOP doing this to your Monstera Deliciosa
349K 4y ago
Kill This Plant
I get the idea behind it. But this is not doing what you think its doing. We need to stop pruning our Monstera Deliciosa. Pruning this ...
4K
Summary
Kill This Plant explains how pruning affects hormone pathways and growth in Monsteras, unlike branching plants like jade. They reveal how cutting stems reverts plants to a juvenile state with smaller leaves and fenestrations.
true
14:42
Now playing
The only 4 Monstera Deliciosa tips you need to know
91K 5mo ago
Kill This Plant
Download My Houseplant App - https://blackthumbplant.app/ Join Our Discord for plant help - https://discord.gg/kw9FnABCRk ...
4K
5 chapters
Find the back of the plant | Measure your light | Get grow lights | Take top cuttings | Conclusion
true
11:01
Now playing
Tips + Tricks For Monstera Deliciosa | ALL YOU NEED TO KNOW 🌿 Indoor Plant Tips For Beginners
47K 5y ago
The Jungle Haven
Monstera deliciosa care tips - swiss cheese plant care - monstera houseplant tips and tricks - indoor plant tips for beginners ...
true
41:52
Now playing
How to continuously grow a large Monstera
146K 1y ago
Sydney Plant Guy
I am chopping my Monstera adansonii in half & re-extend both: one with Moss to grow indoors, one with aroid mix to grow ...
4K
Summary
Sydney Plant Guy demonstrates the 'chop and extend' process to manage a large Monstera adansonii in an indoor setting. The video covers techniques for maintaining plant stability using moss poles and garden stakes while preparing for future growth.
Shorts
5 years of monstera growth! #monstera #plants
696K views
Repotting my NEW indoor monstera plant🪴 #plants #monstera #indoorplants #houseplants #monsteraplant
315K views
New
Converting my M. deliciosa ‘brazil form’ from grown on wood to grown on a moss pole
26K views
Deliciosa VS Borsigiana Monstera | They’re VERY Different - Which One is Yours?
203K views
I have 600 houseplants and my Monstera Deliciosa my absolute FAVORITE plant in my collection 🤍🪴🥹
83K views
true
5:46
Now playing
How to Grow Monstera Deliciosa
6.5K 3mo ago
Tanner The Planter
Day 21/30 of teaching you how to grow your houseplants in a simple way. My reference guide to growing 150 of the most popular ...
Summary
Tanner The Planter provides a comprehensive guide on Monstera deliciosa care, focusing on proper soil composition, effective watering techniques, and optimal lighting conditions. Learn how to encourage healthy root systems and leaf maturity while managing growth through staking.
true
10:15
Now playing
cutting down my GIANT Monstera Deliciosa / Taking Cuttings / Repotting / Feed & Care Nicole's Garden
80K 2y ago
Nicole’s Garden
Please Like and Subscribe! Affiliate links Ponytail sunhat (CAN) https://amzn.to/3UKGeeG Hori Hori Knife (CAN) ...
true
17:59
Now playing
Proper care for Monstera deliciosa! These 7 care tips are a MUST-know 🪴 (Swiss cheese plant)
40K 9mo ago
crazyplantboy
Want your Monstera deliciosa to grow better and healthier? Need an overview of the most important care tips so you can ...
4K
true
2:42
Now playing
239 Days Monstera Deliciosa Time-Lapse – From Seed to Stunning Plant! 🌱
83K 1y ago
Interesting as FCK
Ever wondered how long it really takes to grow a Monstera deliciosa from seed? Here's the full 239-day journey captured in one ...
4K
true
2:52
Now playing
How to Repot Monstera Deliciosa. IT'S EASY!
149K 3y ago
Tanner The Planter
In this video we discuss and demonstrate the general methods of properly re-potting the Monstera Deliciosa. Find Your PERFECT ...
5 chapters
Intro | Preparing the soil | Checking the roots | Repot | Watering
true
19:41
Now playing
Monstera Deliciosa: From Jungle Survivor to Icon 🌿
19K 1y ago
Plantrums
Watch next: The Orchid's Dark Past — Obsession, Mystery, and Survival → https://www.youtube.com/watch?v=q7VpOcKEpAw ...
4 products
7 chapters
Intro | Monstera’s Ancient History | Monstera’s Early Documentation | Monstera’s Biological Secrets | Monstera’s Rise to Fame | Monstera’s Ecological Impact | Monstera’s True Value
Shorts
Eating Monstera Deliciosa Fruit, Finally!
471K views
What to do with your monstera aerial roots? #monsteraplant #plantcaretips #plantcare
981K views
New
Monstera deliciosa (exotic plant that produces fruit) #shorts  #foodforest #tropical  #gardening
1.1K views
A lot of growth already in a year and a half for big mama Monstera Deliciosa. #monsteraplant
433K views
Monstera Deliciosa Repot #monstera #monsteraplant #repotting
49K views
true
11:25
Now playing
How to start Monstera the right way
61K 4mo ago
Kill This Plant
Download My Houseplant App - https://blackthumbplant.app/ Join Our Discord for plant help - https://discord.gg/kw9FnABCRk ...
4K
Summary
Kill This Plant provides a comprehensive guide to maximizing growth for single-stem Monsteras. The discussion covers essential care routines, including light intensity monitoring, selecting appropriate soil amendments to improve drainage, proper staking techniques, and safe fertilizing schedules to help plants thrive.
true
5:37
Now playing
How to Add Support & Prune Back Overgrown Monstera Deliciosa Plants
242K 3y ago
My City Plants
How to prune Monstera Deliciosa (complete guide) ...
5 products
Earns commission
Summary
My City Plants demonstrates techniques for managing large foliage using bamboo stakes, raffia, and clean pruners. Learn how to secure sprawling stems for a narrower profile and selectively cut back growth to maintain a balanced shape.
true
7:13
Now playing
You don’t need to repot your Monstera
80K 2y ago
Kill This Plant
Join our discord community https://discord.gg/RYXzghkuhk * Get 15% off Soltech Grow Lights with promo code: KTP2025 ...
4K
Summary
Kill This Plant explains how to determine when a Monstera actually needs a larger container based on root volume and soil moisture. Learn to identify the right balance of root mass to soil, and discover alternative solutions for managing water retention and light requirements.
true
19:37
Now playing
Saving a Giant: I got an 11+ year old Monstera Deliciosa Large Form
210K 3y ago
Leafy_Luster
MONSTERA DELICIOSA large form exists & I finally found one to call my own. Join me on this Massive Monstera Makeover to ...
4K
true
12:31
Now playing
The Full Story of my 11+ year old Monstera Deliciosa Large Form
108K 2y ago
Leafy_Luster
It's one of my most requested videos to make. THE UPDATE on my rescue Monstera Deliciosa! My rescue mission was quite ...
4K
Shorts
Repotting 7ft+ monstera #shorts #gardenup #monstera #plants
409K views
Monstera Fruit: The Strange Tropical Fruit 🍃🟢
102K views
let's propagate monstera #monstera #propagate #plants #soil #water #roots #areal #cutting #plants
1M views
Grow a BIG Monstera✨ #houseplants #plantcare #plantcaretips #monsteradeliciosa
3.7K views
If you love your monstera, set it free #houseplant #monstera #plantgrowth
351K views
true
14:33
Now playing
MONSTERA PROPAGATION: How to Propagate Monstera deliciosa (2023)
153K 4y ago
Plant Vibrations with Devin Wallien
My favorite Houseplant tools here: https://amzn.to/3sYJ3ta Yesssss, finally I have created a video on how to successfully ...
CC
true
5:00
Now playing
5 quick tips Monstera Deliciosa
88K 5y ago
Swedish Plantguys
In this video we give you 5 quick tips for your Monstera Deliciosa also know as Swiss cheese plant. Enjoy! And thank you for ...
4K
7 chapters
Intro | Juvenile Monstera | Repot | Support | Prune | Humidity | Outro
true
14:29
Now playing
How to Repot a Monstera Deliciosa Plant | Staking | Aerial Roots | Care Guide
37K 3y ago
Ana Perez
Hello friends, here's how to repot a monstera deliciosa plant and how to care for a monstera plant as well! Here are all the ...
4K
CC
12 chapters
intro | When to repot a monstera deliciosa? | What to do when a monstera is really root bound? | What type of potting soil to use for a monstera deliciosa? | What type of pot to use for a monstera? | Are moss poles necessary for a monstera? | What to do with the aerial roots of a monstera | What type of light does a monstera need? | How often should you water a monstera? | How much water should you give a monstera plant? | Fertilizing a monstera plant | Pest Preventions
true
17:27
Now playing
Monstera Deliciosa Single Node Propagation (0-9 Months)
609K 4y ago
Techplant
propagating monstera nodes. In this video I will show you the process of making single node cuttings of monstera deliciosa and ...
1 product
4K
true
9:13
Now playing
5 Things Monsteras HATE That Might Surprise You
387K 3y ago
Sheffield Made Plants
Monsteras aren't particularly fussy plants but there are some key things they hate that might actually surprise you.
Summary
Sheffield Made Plants explains how common care habits like misting, incorrect potting, and allowing dust buildup can harm Monsteras. Learn about the importance of stable humidity, proper root support, and leaf hygiene for optimal plant health and leaf development.
true
11:47
Now playing
MONSTERA DELICIOSA Care | Swiss Cheese Plant Tips & Tricks
187K 7y ago
Good Growing
For bonus content, subscribe to my Patreon ›› https://www.patreon.com/goodgrowing Do you like being trendy? Want plants to ...
11 chapters
Intro | General Info | Water | Light | Temperature | Humidity | Fertiliser | Propagation | Toxicity | Growth Care | Outro
Shorts
Monstera Propagation (My Step by Step Process)
313K views
I CHOPPED UP my Monstera to get it to thrive #houseplants #plants #houseplant #monstera
804K views
Monstera Deliciosa 🌿 Complete Care, Propagation, Height & Fruit Facts!#youtubeshorts#shorts#reels
2.2K views
New
Propagation Loop♻️🌿 #planttips #plants #monstera #propagation #plantcare #diy
18K views
Monstera Deliciosa 2023 Season is Here
1.8M views
true
10:39
Now playing
MONSTERA DELICIOSA CARE + PROPAGATION | indoor swiss cheese plant care
174K 6y ago
Good & Planty
this MONSTER of a care guide is here monstera leaves going yellow? not getting those fenestrations? is your swiss cheese ...
CC
6 chapters
Intro | Water | Light | Repotting | Temperature Humidity | Propagation
true
10:25
Now playing
Monstera richtig pflegen – der komplette Guide | feey 🌱
129K 3y ago
feey
In diesem Video zeigen wir dir alles, was du zur richtigen Pflege deiner Monstera wissen musst: Dazu zählt der richtige Standort, ...
2:46 Das hier ist der Klassiker der Monsteras: die Monstera deliciosa. Diese findest du bei uns im Shop in ganz vielen verschiedenen ...
4K
CC
true
7:35
Now playing
Watch this before you separate your Monstera
38K 1y ago
Kill This Plant
Get 15% off Soltech Grow Lights with promo code: KTP2025 https://glnk.io/0lyv/13 * GET 10% off the best professional ...
4K
Summary
Kill This Plant explains how to properly assess indoor lighting and choose the correct pot size to prevent root rot after propagating. Using a light meter and moisture probe, they demonstrate the relationship between soil volume, oxygen levels, and plant stress.
true
35:40
Now playing
My Monstera Jungle Mint Started Reverting...So I Chopped It All Back 👀
16K 6mo ago
plantsbymelissa
My Monstera jungle mint started reverting, so it was time to cut it back and restart. In this video I'm chopping it down, resetting the ...
4K
8 chapters
Overview | Time to chop | Repotting the base | Chunky pon | Watering | Cuttings | Back in my plant room | Outro
true
15:38
Now playing
Every Type of Monstera Deliciosa Explained (And Which One to Buy)
85K 2y ago
Kaylee Ellen
M E R C H : https://kayleeellen.creator-spring.com/? Monstera Vareities: ...
Summary
Kaylee Ellen outlines the differences between small and large form Monstera deliciosa, focusing on internodal spacing, growth patterns, and leaf characteristics. Learn which variety best suits your available space and aesthetic preferences through a detailed comparison of popular green and variegated cultivars.
Shorts
The MOST DANGEROUS Fruit? - The Monstera Deliciosa Fruit!
229K views
MONSTERA Propagation with 100% SUCCESS | Watch Me Turn a sad 😢 Plant into 11 Happy 🤓babies!
607K views
Monstera Deliciosa : The Delicious Monster Houseplant ! 🪴🍍#garden  #farming #organic
3.1M views
Monstera deliciosa fruit #asmr
901K views
Would you do this?! Full Monstera Cut Back!
456K views
true
9:13
Now playing
5 Things Monsteras HATE That Might Surprise You
387K 3y ago
Sheffield Made Plants
Monsteras aren't particularly fussy plants but there are some key things they hate that might actually surprise you.
Summary
Sheffield Made Plants explains how common care habits like misting, incorrect potting, and allowing dust buildup can harm Monsteras. Learn about the importance of stable humidity, proper root support, and leaf hygiene for optimal plant health and leaf development.
true
11:00
Now playing
How to water propagate a Monstera Deliciosa
64K 1y ago
Kill This Plant
Download My Houseplant App - https://blackthumbplant.app/ Join Our Discord for plant help - https://discord.gg/kw9FnABCRk ...
4K
Summary
Kill This Plant demonstrates how to take top and mid cuttings for propagation while ensuring healthy root development and foliage maintenance. Learn techniques to prevent rot, choose appropriate pot sizes based on root mass, and mix suitable soil for growth.
Shorts
Monstera deliciosa harvest!
4.4M views
Here’s 4 tips to grow a HUGE Monstera Deliciosa! 🌱🪴
286K views
The Secret, Edible Fruit Hiding on Your Houseplant #monstera #animation
713K views
Monstera deliciosa propagation in water❤️#shorts #youtubeshorts #like #viral #fyp
425K views
Monstera Deliciosa Fruit #shorts
51K views
true
2:41
Now playing
My Mutant Monstera Deliciosa Update
7.9K 2y ago
Techplant
Amazon Affiliate link to light https://amzn.to/3S7wJVN These are amazon affiliate links, they don't change the price, however after ...
4K
true
0:47
Now playing
Monstera Deliciosa Time Lapse (HUGE LEAF)
28K 3y ago
Interesting as FCK
In this plant time lapse, we'll be taking a look at a GIANT monstera deliciosa growing, one of my favorite plants in the world!
4K
true
8:16
Now playing
Monstera Deliciosa LARGE FORM: Repotting & Root Dividing
136K 6y ago
reallybigplant
How to repot an extra large Monstera Deliciosa with a huge developed root system! Join me while I try to root divide this Monstera ...
true
8:16
Now playing
How to Grow and Care for Monstera (Swiss Cheese) Indoors
615K 5y ago
Garden Up
LINKS| Good quality soil : https://gardenup.in/products/ready-potting-soil-mix Succulent soil ...
true
7:41
Now playing
Monstera Roots - 4 Things to do with Aerial Roots - Monstera deliciosa (2023)
189K 3y ago
Plant Vibrations with Devin Wallien
Shop plants here! https://plantvibrations-shop.myshopify.com/ While we're growing our Monstera deliciosa or many other aroid ...
CC
Summary
Devin Wallien explains how aerial roots function as nutrient and water absorbers for the plant. The video demonstrates practical techniques for managing these roots, including guiding them into a moss pole, burying them in the soil, or placing them in water to enhance growth.
true
12:09
Now playing
The Easiest Monsteras to Care For 🌿 Great for Beginners Too!
3K 3mo ago
planty skies
🌿 Monsteras are among the most popular houseplants – but which varieties are truly easy to care for and suitable for beginners ...
4K
true
6:11
Now playing
The Giant Monstera Deliciosa Node Update
12K 1y ago
Techplant
Here are the giant monstera nodes I've had propagating for 2 yeara These are amazon affiliate links, they don't change the price, ...
4K
Summary
Techplant documents a two-year propagation experiment using various methods to revive severely rotted monstera cuttings. Despite initial setbaks and neglect, the video tracks the long-term survival and growth of the nodes under different environmental conditions.
true
2:44
Now playing
How to safely eat the poisonous tropical fruit Monstera Deliciosa
130K 9y ago
SleepyLizard
One of our tastiest tropical fruits is also our most dangerous. The Monstera Deliciousa, known as the "fruit salad fruit" combines ...
Shorts
Look at all that Monstera Deliciosa #fruit  #art #india #photogr #reels #bhfyp #explorepage #garden
1.2M views
"Root rot almost killed Monstera… but I brought it back" #plants #gardening
481K views
Monstera Deliciosa Fruit!!!
50K views
Tips to keep your monstera deliciosa alive #plantcare #monsteraplant #monstera
1.1M views
😲 8FT TALL Monstera Deliciosa Repot 🌿 PART 1 #monstera #houseplants #indoorplants #repotting #plants
13K views
true
8:57
Now playing
Monstera Deliciosa Review - Weird Fruit Explorer - Ep. 108
303K 11y ago
Weird Explorer
Help support my series by checking out: My Patreon Page: https://www.patreon.com/weirdexplorer ---- Or get a TSHIRT: ...
CC
true
3:25
Now playing
Monstera deliciosa learns its care easily
122K 3y ago
El De Las Plantas
In this video, I'll show you a quick and easy way to keep your Monstera Deliciosa healthy and happy. #plants ...
4K
true
4:11
Now playing
How to Set Your Monstera For Success // Monstera Deliciosa Plant Care
38K 3y ago
Plantalogy
2 mistakes can hinder the growth of your monstera. These are some common mistakes plant growers often do but rarely notice ...
Summary
Plantalogy highlights two common mistakes regarding pot size and planting depth that lead to root rot and yellowing leaves. Learn proper repotting techniques and how to provide adequate vertical support for a healthy, lush plant.
true
5:47
Now playing
Monstera Plant Care for Beginners (Do This First!)
10K 1y ago
The Girl with a Shovel
Keywords: monstera houseplant, monstera deliciosa care, monstera plant care indoors, how to support monstera plant, saving a ...
CC
7 chapters
Intro | Light Requirements | Watering Requirements | Soil Requirements | Fertilizer Requirements | Humidity Requirements | Bonus Tips
true
20:37
Now playing
The Tastiest Fruit In The World! - How to Pick and Eat Monstera Deliciosa Safely
18K 1y ago
Fruitful Trees
Tom at www.Guacfarm.com shows us how to pick and eat the Monstera Deliciosa Fruit safely. He also teaches me how to correctly ...
true
30:30
Now playing
Monstera Deliciosa Types! | Variegated and Non-Variegated
61K 4y ago
Kaylee Ellen
Head to https://squarespace.com/kayleeellen to save 10% off your first purchase of a website or domain using code ...
Shorts
monstera leaves not splitting? do this! 🌱
382K views
I did the big monstera deliciosa chop… now it’s time to propagate! #monsteraplant #propagation
391K views
Look at how BIG this pretty Monstera is! #plants #monsteradeliciosa #plantmom #propagation
110K views
How To Propagate A Monstera Deliciosa #propagation #monsteradeliciosa #plantcaretips #plantcare
468K views
Repotting my very rootbound Monstera Thai constellation #plantcare #monsterathaiconstellation
661K views
true
5:21
Now playing
HOUSEPLANT CARE TIPS | HOW TO CARE FOR MONSTERA ADANSONII SWISS CHEESE
77K 4y ago
Succulents Box
In this video, you'll find all the care tips for Monstera Swiss Cheese. Time code 0:00 Intro 0:39 Lighting 0:59 Watering 1:41 ...
10 chapters
Intro | Lighting | Watering | Temperature | Soil | Pruning | Propagation | Troubleshooting | Toxicity | Outro
true
4:01
Now playing
Monstera Rehab. How to properly prune your Monstera deliciosa 🪴
235K 4y ago
Paul’s & Sandy’s Too 🪴Karens Content 🪴
Rehab pruning a damaged Monstera! What you need to know.
Summary
After a shipment arrived with heat-damaged plants, Karen demonstrates how to rescue a Monstera deliciosa. The tutorial focuses on identifying the proper spot to prune without removing future leaf growth, a technique applicable to other plants with similar growth patterns.
true
11:32
Now playing
I Stopped Killing Monsteras Once I Knew This
25K 10mo ago
Sheffield Made Plants
Head to https://squarespace.com/sheffield to save 10% off your first purchase of a website or domain using code SHEFFIELD ...
4K
Summary
Learn how to optimize lighting and watering for faster leaf growth and more holes in Monstera plants. Sheffield Made Plants covers essential potting media, managing aerial roots, and troubleshooting pruning techniques.
true
13:59
Now playing
Monstera Deliciosa Care: 7 Things That Keep It Happy & Growing
251 3w ago
Her green space
Monstera deliciosa is one of the easiest houseplants to grow once you understand what it really needs — but small leaves, ...
30 products
15 chapters
Monstera Deliciosa Care | How much light does Monstera need? | Why your Monstera has small leaves or no fenestrations | How I water my Monstera | Best soil mix for Monstera | When should you repot? | Does Monstera really need a moss pole? | What to do with Monstera aerial roots | How much humidity does Monstera need? | Fertilizing Monstera | My Monstera growing in LECA | Yellow leaves, brown leaves & common problems | Cleaning leaves & checking for pests | Pests | What keeps A monstera happy
true
12:10
Now playing
MONSTERA DELICIOSA CARE TIPS
152K 6y ago
Green Yard TV
Monstera Deliciosa Care tips I hope you find this video helpful and interesting! If you did, please don't forget to hit LIKE, ...
true
10:39
Now playing
How to properly care for Monstera Thai Constellation – Everything you need to know! 🌿✨
20K 1y ago
planty skies
🌿✨ The Monstera deliciosa Thai Constellation is one of the most beautiful and sought-after houseplants – but how do you care ...
4K
true
9:47
Now playing
Everything You Need to Know About MONSTERA DELICIOSA | Monstera deliciosa plant care | Bagicha Vl...
34K 4y ago
Drakht
The Video Content is not intended to be a substitute for professional medical advice, diagnosis, or treatment. Always seek the ...
true
16:24
Now playing
Everything about MONSTERA DELICIOSA | Growth and care guide
175K 4y ago
I Giardini di Ellis
► Read my article on Monstera: https://igiardinidiellis.it/monstera-deliciosa/ ► Support your plants https://www.amazon.it ...
CC
9 chapters
Intro | PROVENIENZA e nome della Monstera deliciosa | LUCE e temperatura ideale per la Monstera deliciosa | ACQUA e umidità ideale per la Monstera deliciosa | RINVASI e terriccio per la Monstera deliciosa | PARTICOLARITÀ della Monstera deliciosa | CURA per la Monstera deliciosa | PROPAGAZIONE per la Monstera deliciosa | MEGLIO SAPERLO
true
11:46
Now playing
The Alternative Way To Grow The Best Monstera
79K 2y ago
Sheffield Made Plants
Head to https://squarespace.com/sheffield to save 10% off your first purchase of a website or domain using code SHEFFIELD ...
4K
8 chapters
Intro | Root Rot | Top cuttings | Vase | Stones | Assembling | Watering | pH
true
12:21
Now playing
The truth about monstera care!!!!
35K 1y ago
addictoplantes
Everything you need to know about caring for Monsteras! Watering, lighting, repotting, pests... I'll tell you everything you ...
true
16:37
Now playing
Meet All Variegated Monstera Varieties And Expert Care Tips @noldy.topan
115K 3y ago
OnlyPlants
In this tour, we cover the most extensive variegated monstera collector in Indonesia - Mr Noldy Topan of @dutahijauabadi.
6:55 ... the monstera deliciosa called the Sphinx Mr Noli got this from Thailand and it's well known for its sectoral variegation this means ...
4K
true
26:17
Now playing
Airlayering, chopping, and repotting my EXTRA large Monstera Deliciosa!
80K 2y ago
PlantswithKrystal
Hey hey! Today's video shows the process of air layering, chopping, and repotting my large monstera. Hope it gives you a little ...
1 product
Summary
After successfully air layering a massive Monstera Deliciosa, Krystal proceeds with a daring chop to manage its height. The process involves transferring the propagated cutting into a new, larger pot alongside the original plant to create a fuller appearance.
true
1:12
Now playing
Comment entretenir ma monstera ? - Truffaut
70K 7y ago
TRUFFAUT
Découvrez le Monstera et ses secrets d'entretien ! Plante iconique des intérieurs végétalisés, le Monstera Deliciosa, aussi ...
true
18:15
Now playing
Monstera deliciosa: Easy propagation and tips for making it grow beautifully.
178K 3y ago
Ivet Suculentas y mas
Hello plant lovers! This is your friend Ivet, from Oaxaca, who is completely in love with plants. My little paradise is ...
4K
Summary
Ivet muestra cómo propagar una monstera deliciosa para recuperar su color blanco característico, utilizando un sustrato ligero a base de turba de coco y perlita. La demostración incluye el corte estratégico del tallo y el uso de tutores para asegurar raíces fuertes y frondosidad.
true
20:45
Now playing
Why Is My Variegated Monstera Reverting? (Causes & How to Fix It!)
2.4K 8mo ago
Jamie Rootledge
Is your variegated Monstera reverting back to green? In this video, I break down why variegated Monsteras (Albo, Thai ...
Summary
Jamie Rootledge demonstrates how to propagate a reverted jungle mint Monstera to encourage the return of variegated growth. The process involves identifying healthy nodes on the stem, making strategic cuttings, and establishing them in moss to foster new, colorful foliage.
true
4:30
Now playing
Costilla de Adán o Monstera Deliciosa: características y reproducción - Decogarden - Jardinatis
84K 6y ago
Jardinatis
Iñigo Segurola nos muestra las enormes variedades de Costilla de Adán que tiene en su invernadero y nos enseña las ...
25 videos
Monstera Essentials
Kill This Plant
Playlist
How to set up your Monstera for success · 3:58
why people keep Killing their Monstera · 3:13
View full playlist
If your Monstera Deliciosa is struggling, check out these essential videos to help troubleshoot your problems.
true
13:15
Now playing
Your Monstera Will NEVER Thrive Unless You Do This
141K 1y ago
Sheffield Made Plants
Head to https://squarespace.com/sheffield to save 10% off your first purchase of a website or domain using code SHEFFIELD ...
4K
true
42:26
Now playing
My Variegated Monstera Deliciosa Collection 🌿
30K 1y ago
plantsbymelissa
In this video, I'm sharing a closer look at all the stunning variegated monstera deliciosas in my collection, including albo, aurea, ...
4K
9 chapters
intro | Monstera Deliciosa ‘Mint’ (NOID, non-TC) | Monstera Deliciosa ‘Jungle Mint’ (NOID, non-TC) | Monstera Deliciosa ‘Albo Variegata’ (White Tiger type) | Monstera Deliciosa ‘Mint’ (tissue culture) | Monstera Deliciosa ‘Aurea’ | Monstera Deliciosa ‘Thai Constellation’ | Monstera Deliciosa ‘Albo Variegata’ | outro
true
17:59
Now playing
Monstera Mania! | All of the Monstera Deliciosa I am Growing!
3.1K 6mo ago
PlantswithJacque
Monstera Deliciosa Types Showcase In this video, I'm walking you through my collection of Monstera deliciosa varieties and ...
Summary
PlantswithJacque showcases a diverse collection of Monstera deliciosa varieties, detailing unique variegation patterns and specific growth forms. Each plant is presented with insights into its care requirements, preferred growing conditions, and individual developmental stages to help viewers distinguish between these popular botanical forms.
true
10:03
Now playing
The Ultimate Guide to Monstera Care | How to Grow Big, Healthy Monstera Plants Indoors
7.8K 1y ago
The Cozy Jungle
Whether you're a beginner or an experienced plant parent, this video will help you keep your Monstera deliciosa looking vibrant ...
4K
true
4:56
Now playing
Monstera deliciosa (Swiss Cheese Plant) Care 🌿 Watering, Propagation, Pruning and more!
118K 5y ago
EcologíaVerde
🌿 Are you thinking about growing a Monstera deliciosa (Swiss cheese plant) indoors? In this Green Ecology video, we bring you ...
7 chapters
Cuidados de la costilla de Adán o Monstera deliciosa | Temperatura adecuada | Riego de la Monstera deliciosa | Luz y ubicación ideales | Poda recomendada | Hojas amarillas en la Monstera deliciosa | Cómo reproducir la planta costilla de Adán
true
1:09:24
Now playing
Growing & chopping up my huge Monstera adansonii (includes 12 months of updates)
31K 1y ago
Sydney Plant Guy
Let's grow my Monstera adansonii even larger over the course of a full year! watch my review of the grow light: ...
4K
Summary
Sydney Plant Guy tracks the recovery of a Monstera adansonii after an aggressive pruning session. The journey includes restructuring the plant's support pole and handling unexpected flowering to manage growth over the course of a full year.
true
30:59
Now playing
$15 BIG Monstera Deliciosa at 50% OFF HOME DEPOT PLANT SALE! Prices, Pet Safe Plants, Rare Plants
15K 2mo ago
Plant Life with Ashley Anita
plants #homedepotfinds #plantshopping *This is a good vibes only channel for all plant friends to enjoy! :) Any negative and ...
true
8:16
Now playing
Monstera Deliciosa LARGE FORM: Repotting & Root Dividing
136K 6y ago
reallybigplant
How to repot an extra large Monstera Deliciosa with a huge developed root system! Join me while I try to root divide this Monstera ...
true
8:16
Now playing
How to Grow and Care for Monstera (Swiss Cheese) Indoors
615K 5y ago
Garden Up
LINKS| Good quality soil : https://gardenup.in/products/ready-potting-soil-mix Succulent soil ...
true
7:41
Now playing
Monstera Roots - 4 Things to do with Aerial Roots - Monstera deliciosa (2023)
189K 3y ago
Plant Vibrations with Devin Wallien
Shop plants here! https://plantvibrations-shop.myshopify.com/ While we're growing our Monstera deliciosa or many other aroid ...
CC
Summary
Devin Wallien explains how aerial roots function as nutrient and water absorbers for the plant. The video demonstrates practical techniques for managing these roots, including guiding them into a moss pole, burying them in the soil, or placing them in water to enhance growth.
true
12:09
Now playing
The Easiest Monsteras to Care For 🌿 Great for Beginners Too!
3K 3mo ago
planty skies
🌿 Monsteras are among the most popular houseplants – but which varieties are truly easy to care for and suitable for beginners ...
4K
true
6:11
Now playing
The Giant Monstera Deliciosa Node Update
12K 1y ago
Techplant
Here are the giant monstera nodes I've had propagating for 2 yeara These are amazon affiliate links, they don't change the price, ...
4K
Summary
Techplant documents a two-year propagation experiment using various methods to revive severely rotted monstera cuttings. Despite initial setbaks and neglect, the video tracks the long-term survival and growth of the nodes under different environmental conditions.
true
2:44
Now playing
How to safely eat the poisonous tropical fruit Monstera Deliciosa
130K 9y ago
SleepyLizard
One of our tastiest tropical fruits is also our most dangerous. The Monstera Deliciousa, known as the "fruit salad fruit" combines ...
Shorts
Look at all that Monstera Deliciosa #fruit  #art #india #photogr #reels #bhfyp #explorepage #garden
1.2M views
"Root rot almost killed Monstera… but I brought it back" #plants #gardening
481K views
Monstera Deliciosa Fruit!!!
50K views
Tips to keep your monstera deliciosa alive #plantcare #monsteraplant #monstera
1.1M views
😲 8FT TALL Monstera Deliciosa Repot 🌿 PART 1 #monstera #houseplants #indoorplants #repotting #plants
13K views
true
8:57
Now playing
Monstera Deliciosa Review - Weird Fruit Explorer - Ep. 108
303K 11y ago
Weird Explorer
Help support my series by checking out: My Patreon Page: https://www.patreon.com/weirdexplorer ---- Or get a TSHIRT: ...
CC
true
3:25
Now playing
Monstera deliciosa learns its care easily
122K 3y ago
El De Las Plantas
In this video, I'll show you a quick and easy way to keep your Monstera Deliciosa healthy and happy. #plants ...
4K
true
4:11
Now playing
How to Set Your Monstera For Success // Monstera Deliciosa Plant Care
38K 3y ago
Plantalogy
2 mistakes can hinder the growth of your monstera. These are some common mistakes plant growers often do but rarely notice ...
Summary
Plantalogy highlights two common mistakes regarding pot size and planting depth that lead to root rot and yellowing leaves. Learn proper repotting techniques and how to provide adequate vertical support for a healthy, lush plant.
true
5:47
Now playing
Monstera Plant Care for Beginners (Do This First!)
10K 1y ago
The Girl with a Shovel
Keywords: monstera houseplant, monstera deliciosa care, monstera plant care indoors, how to support monstera plant, saving a ...
CC
7 chapters
Intro | Light Requirements | Watering Requirements | Soil Requirements | Fertilizer Requirements | Humidity Requirements | Bonus Tips
true
20:37
Now playing
The Tastiest Fruit In The World! - How to Pick and Eat Monstera Deliciosa Safely
18K 1y ago
Fruitful Trees
Tom at www.Guacfarm.com shows us how to pick and eat the Monstera Deliciosa Fruit safely. He also teaches me how to correctly ...
true
30:30
Now playing
Monstera Deliciosa Types! | Variegated and Non-Variegated
61K 4y ago
Kaylee Ellen
Head to https://squarespace.com/kayleeellen to save 10% off your first purchase of a website or domain using code ...
Shorts
monstera leaves not splitting? do this! 🌱
382K views
I did the big monstera deliciosa chop… now it’s time to propagate! #monsteraplant #propagation
391K views
Look at how BIG this pretty Monstera is! #plants #monsteradeliciosa #plantmom #propagation
110K views
How To Propagate A Monstera Deliciosa #propagation #monsteradeliciosa #plantcaretips #plantcare
468K views
Repotting my very rootbound Monstera Thai constellation #plantcare #monsterathaiconstellation
661K views
true
5:21
Now playing
HOUSEPLANT CARE TIPS | HOW TO CARE FOR MONSTERA ADANSONII SWISS CHEESE
77K 4y ago
Succulents Box
In this video, you'll find all the care tips for Monstera Swiss Cheese. Time code 0:00 Intro 0:39 Lighting 0:59 Watering 1:41 ...
10 chapters
Intro | Lighting | Watering | Temperature | Soil | Pruning | Propagation | Troubleshooting | Toxicity | Outro
true
4:01
Now playing
Monstera Rehab. How to properly prune your Monstera deliciosa 🪴
235K 4y ago
Paul’s & Sandy’s Too 🪴Karens Content 🪴
Rehab pruning a damaged Monstera! What you need to know.
Summary
After a shipment arrived with heat-damaged plants, Karen demonstrates how to rescue a Monstera deliciosa. The tutorial focuses on identifying the proper spot to prune without removing future leaf growth, a technique applicable to other plants with similar growth patterns.
true
11:32
Now playing
I Stopped Killing Monsteras Once I Knew This
25K 10mo ago
Sheffield Made Plants
Head to https://squarespace.com/sheffield to save 10% off your first purchase of a website or domain using code SHEFFIELD ...
4K
Summary
Learn how to optimize lighting and watering for faster leaf growth and more holes in Monstera plants. Sheffield Made Plants covers essential potting media, managing aerial roots, and troubleshooting pruning techniques.
true
13:59
Now playing
Monstera Deliciosa Care: 7 Things That Keep It Happy & Growing
251 3w ago
Her green space
Monstera deliciosa is one of the easiest houseplants to grow once you understand what it really needs — but small leaves, ...
30 products
15 chapters
Monstera Deliciosa Care | How much light does Monstera need? | Why your Monstera has small leaves or no fenestrations | How I water my Monstera | Best soil mix for Monstera | When should you repot? | Does Monstera really need a moss pole? | What to do with Monstera aerial roots | How much humidity does Monstera need? | Fertilizing Monstera | My Monstera growing in LECA | Yellow leaves, brown leaves & common problems | Cleaning leaves & checking for pests | Pests | What keeps A monstera happy
true
12:10
Now playing
MONSTERA DELICIOSA CARE TIPS
152K 6y ago
Green Yard TV
Monstera Deliciosa Care tips I hope you find this video helpful and interesting! If you did, please don't forget to hit LIKE, ...
true
10:39
Now playing
How to properly care for Monstera Thai Constellation – Everything you need to know! 🌿✨
20K 1y ago
planty skies
🌿✨ The Monstera deliciosa Thai Constellation is one of the most beautiful and sought-after houseplants – but how do you care ...
4K
true
9:47
Now playing
Everything You Need to Know About MONSTERA DELICIOSA | Monstera deliciosa plant care | Bagicha Vl...
34K 4y ago
Drakht
The Video Content is not intended to be a substitute for professional medical advice, diagnosis, or treatment. Always seek the ...
true
16:24
Now playing
Everything about MONSTERA DELICIOSA | Growth and care guide
175K 4y ago
I Giardini di Ellis
► Read my article on Monstera: https://igiardinidiellis.it/monstera-deliciosa/ ► Support your plants https://www.amazon.it ...
CC
9 chapters
Intro | PROVENIENZA e nome della Monstera deliciosa | LUCE e temperatura ideale per la Monstera deliciosa | ACQUA e umidità ideale per la Monstera deliciosa | RINVASI e terriccio per la Monstera deliciosa | PARTICOLARITÀ della Monstera deliciosa | CURA per la Monstera deliciosa | PROPAGAZIONE per la Monstera deliciosa | MEGLIO SAPERLO
true
11:46
Now playing
The Alternative Way To Grow The Best Monstera
79K 2y ago
Sheffield Made Plants
Head to https://squarespace.com/sheffield to save 10% off your first purchase of a website or domain using code SHEFFIELD ...
4K
8 chapters
Intro | Root Rot | Top cuttings | Vase | Stones | Assembling | Watering | pH
true
12:21
Now playing
The truth about monstera care!!!!
35K 1y ago
addictoplantes
Everything you need to know about caring for Monsteras! Watering, lighting, repotting, pests... I'll tell you everything you ...
true
16:37
Now playing
Meet All Variegated Monstera Varieties And Expert Care Tips @noldy.topan
115K 3y ago
OnlyPlants
In this tour, we cover the most extensive variegated monstera collector in Indonesia - Mr Noldy Topan of @dutahijauabadi.
6:55 ... the monstera deliciosa called the Sphinx Mr Noli got this from Thailand and it's well known for its sectoral variegation this means ...
4K
true
26:17
Now playing
Airlayering, chopping, and repotting my EXTRA large Monstera Deliciosa!
80K 2y ago
PlantswithKrystal
Hey hey! Today's video shows the process of air layering, chopping, and repotting my large monstera. Hope it gives you a little ...
1 product
Summary
After successfully air layering a massive Monstera Deliciosa, Krystal proceeds with a daring chop to manage its height. The process involves transferring the propagated cutting into a new, larger pot alongside the original plant to create a fuller appearance.
true
1:12
Now playing
Comment entretenir ma monstera ? - Truffaut
70K 7y ago
TRUFFAUT
Découvrez le Monstera et ses secrets d'entretien ! Plante iconique des intérieurs végétalisés, le Monstera Deliciosa, aussi ...
true
18:15
Now playing
Monstera deliciosa: Easy propagation and tips for making it grow beautifully.
178K 3y ago
Ivet Suculentas y mas
Hello plant lovers! This is your friend Ivet, from Oaxaca, who is completely in love with plants. My little paradise is ...
4K
Summary
Ivet muestra cómo propagar una monstera deliciosa para recuperar su color blanco característico, utilizando un sustrato ligero a base de turba de coco y perlita. La demostración incluye el corte estratégico del tallo y el uso de tutores para asegurar raíces fuertes y frondosidad.
true
20:45
Now playing
Why Is My Variegated Monstera Reverting? (Causes & How to Fix It!)
2.4K 8mo ago
Jamie Rootledge
Is your variegated Monstera reverting back to green? In this video, I break down why variegated Monsteras (Albo, Thai ...
Summary
Jamie Rootledge demonstrates how to propagate a reverted jungle mint Monstera to encourage the return of variegated growth. The process involves identifying healthy nodes on the stem, making strategic cuttings, and establishing them in moss to foster new, colorful foliage.
true
4:30
Now playing
Costilla de Adán o Monstera Deliciosa: características y reproducción - Decogarden - Jardinatis
84K 6y ago
Jardinatis
Iñigo Segurola nos muestra las enormes variedades de Costilla de Adán que tiene en su invernadero y nos enseña las ...
25 videos
Monstera Essentials
Kill This Plant
Playlist
How to set up your Monstera for success · 3:58
why people keep Killing their Monstera · 3:13
View full playlist
If your Monstera Deliciosa is struggling, check out these essential videos to help troubleshoot your problems.
true
13:15
Now playing
Your Monstera Will NEVER Thrive Unless You Do This
141K 1y ago
Sheffield Made Plants
Head to https://squarespace.com/sheffield to save 10% off your first purchase of a website or domain using code SHEFFIELD ...
4K
true
42:26
Now playing
My Variegated Monstera Deliciosa Collection 🌿
30K 1y ago
plantsbymelissa
In this video, I'm sharing a closer look at all the stunning variegated monstera deliciosas in my collection, including albo, aurea, ...
4K
9 chapters
intro | Monstera Deliciosa ‘Mint’ (NOID, non-TC) | Monstera Deliciosa ‘Jungle Mint’ (NOID, non-TC) | Monstera Deliciosa ‘Albo Variegata’ (White Tiger type) | Monstera Deliciosa ‘Mint’ (tissue culture) | Monstera Deliciosa ‘Aurea’ | Monstera Deliciosa ‘Thai Constellation’ | Monstera Deliciosa ‘Albo Variegata’ | outro
true
17:59
Now playing
Monstera Mania! | All of the Monstera Deliciosa I am Growing!
3.1K 6mo ago
PlantswithJacque
Monstera Deliciosa Types Showcase In this video, I'm walking you through my collection of Monstera deliciosa varieties and ...
Summary
PlantswithJacque showcases a diverse collection of Monstera deliciosa varieties, detailing unique variegation patterns and specific growth forms. Each plant is presented with insights into its care requirements, preferred growing conditions, and individual developmental stages to help viewers distinguish between these popular botanical forms.
true
10:03
Now playing
The Ultimate Guide to Monstera Care | How to Grow Big, Healthy Monstera Plants Indoors
7.8K 1y ago
The Cozy Jungle
Whether you're a beginner or an experienced plant parent, this video will help you keep your Monstera deliciosa looking vibrant ...
4K
true
4:56
Now playing
Monstera deliciosa (Swiss Cheese Plant) Care 🌿 Watering, Propagation, Pruning and more!
118K 5y ago
EcologíaVerde
🌿 Are you thinking about growing a Monstera deliciosa (Swiss cheese plant) indoors? In this Green Ecology video, we bring you ...
7 chapters
Cuidados de la costilla de Adán o Monstera deliciosa | Temperatura adecuada | Riego de la Monstera deliciosa | Luz y ubicación ideales | Poda recomendada | Hojas amarillas en la Monstera deliciosa | Cómo reproducir la planta costilla de Adán
true
1:09:24
Now playing
Growing & chopping up my huge Monstera adansonii (includes 12 months of updates)
31K 1y ago
Sydney Plant Guy
Let's grow my Monstera adansonii even larger over the course of a full year! watch my review of the grow light: ...
4K
Summary
Sydney Plant Guy tracks the recovery of a Monstera adansonii after an aggressive pruning session. The journey includes restructuring the plant's support pole and handling unexpected flowering to manage growth over the course of a full year.
true
30:59
Now playing
$15 BIG Monstera Deliciosa at 50% OFF HOME DEPOT PLANT SALE! Prices, Pet Safe Plants, Rare Plants
15K 2mo ago
Plant Life with Ashley Anita
plants #homedepotfinds #plantshopping *This is a good vibes only channel for all plant friends to enjoy! :) Any negative and ...
true

**Feature evidence last updated:** 2026-10-22.

The shared assessment status line and revision history are current upstream; this dated section records the submission's feature evidence and revisions without rescoring upstream or claiming a new combined build or runtime measurement.

- Current-main conflict reconciliation retains upstream and feature behavior; scores and estimates unchanged
- Superseded 2026-10-20 behavior, retained as history: Artboard Options shares existing presets and orientation controls and can fit artwork without moving it; artwork fitting uses the original target artboard and revision-scoped bounds caches

- Artboard Options puts Fit to Artwork Bounds and Fit to Selected Art in the Preset menu. Artwork bounds cover all drawn document art, including other artboards and off-board art; selection bounds may span artboards. Fit changes only the active artboard and keeps artwork stationary, including position-only fitting with Move Artwork enabled. Existing Background Color and presets remain available; scores and estimates unchanged.

Artboard Options forwards the active Artboard tool's Scale Artwork with Artboard option for ordinary resizing. Fit to Artwork Bounds and Fit to Selected Art explicitly leave artwork stationary. Dialog regression coverage checks preset/orientation/typed-size intent with scaling on and off, draft cancellation, one-step Undo/Redo, and fitting while Move Artwork and Scale Artwork are enabled. This is a narrow follow-up to #3277; no parity score or estimate is reassessed.

### Feature revisions

| Date | Change | Summary |
|---|---|---|
| 2026-10-22 | minor | Reconcile upstream documentation after review F-3; preserve every dated revision and the original scaleArt runtime evidence without repeating completed jobs. |
| 2026-10-22 | minor | Reconcile upstream documentation after review F-2; preserve every dated revision and the original scaleArt runtime evidence without repeating completed jobs. |
| 2026-10-22 | minor | Reconcile subsequent upstream progress updates without changing the runtime or repeating completed evidence; retain every dated upstream and feature revision. |
| 2026-10-22 | minor | Resynchronize newer upstream progress documentation using the same owned source/build cache; preserve every feature revision and the original scaleArt runtime evidence. |
| 2026-10-22 | minor | Retain the latest upstream shared assessment and complete revision history after newer documentation landed; scaleArt runtime and its original evidence remain unchanged. |
| 2026-10-22 | minor | Correct the scaleArt follow-up evidence description to match current upstream shared status and revision history; runtime, evidence, scores and estimates unchanged. |
| 2026-10-22 | minor | Artboard Options honors Scale Artwork with Artboard for preset, orientation and typed-size changes; both Fits keep artwork stationary; scores and estimates unchanged. |
| 2026-10-21 | minor | Correct current global-all-art and document/revision cache descriptions; mark target-scoped fitting as superseded 2026-10-20 history. Runtime, tests, scores and estimates unchanged. |
| 2026-10-21 | minor | Artboard Options puts Fit to Artwork Bounds and Fit to Selected Art in the Preset menu. Artwork bounds cover all drawn document art, including other artboards and off-board art; selection bounds may span artboards. Fit changes only the active artboard and keeps artwork stationary, including position-only fitting with Move Artwork enabled. Existing Background Color and presets remain available; scores and estimates unchanged. |
| 2026-10-21 | minor | Reconcile feature status and revision evidence in a dated section; retain shared baseline metadata for independent upstream updates. Scores and estimates are not reassessed. |
| 2026-10-21 | minor | Reconcile this feature with current main, preserving both sets of behavior and progress evidence; scores and estimates unchanged. |
| 2026-10-20 | minor | Review follow-up: artwork fitting uses the original target artboard and revision-scoped bounds caches; focused regression evidence and behavior documentation updated; scores and hours unchanged. |
| 2026-10-20 | minor | Artboard Options shares existing presets and orientation controls and can fit artwork without moving it; scores and hours unchanged. |

How VectorCraft's tools, handles, snapping and panels behave next to Illustrator's. A vector illustrator's
trust is in the details: where a handle lands, what Shift and Alt do mid-drag, what a smart guide snaps to,
how far an arrow key nudges. This checklist is the work list for the **interaction-fidelity pass**
([gaps.md](gaps.md) G1). Part of [target-app-parity.md](target-app-parity.md).

**How it is measured:** from the tool state machines' own documentation and tests (`crates/tools/src/*.rs`,
`crates/ui-egui/src/canvas.rs`, `shortcuts.rs`, the `tests_*` files) against Illustrator's public documentation
(helpx tool pages and the keyboard-shortcut reference). Never against the running app: the clean-room rule
forbids launching or measuring Illustrator, so there has been **no side-by-side session**. Every "done" below is
"behaves as the documentation describes", which is weaker than "a power user can't tell". Status:
**done** (documented behaviour implemented and tested), **partial**, **missing**.

## Summary

| Area | % | Remaining (h) | Kind |
|---|---:|---:|---|
| Pen, Curvature and anchor tools | 85% | 6–10 | estimated |
| Direct Selection, handles and anchors | 85% | 5–8 | estimated |
| Smart guides and snapping | 70% | 8–12 | estimated |
| Transform handles, bounding box, Free Transform | 80% | 5–8 | estimated |
| Nudging, numeric entry, precision, units | 85% | 3–5 | estimated |
| Modifier keys and temporary tools | 80% | 5–8 | estimated |
| Keyboard shortcuts (default set) | 70% | 6–10 | estimated |
| Cursors | 75% | 3–5 | estimated |
| Panels, Control bar, Properties, context menus | 70% | 15–25 | estimated |
| Look and feel (layout, theme, metrics) | 80% | 5–10 | measured once against public screenshots on 2026-10-02; panels added since not re-measured |
| Accessibility (screen reader, keyboard-only canvas) | 20% | 15–25 | estimated: AccessKit on by default for egui widgets; canvas and custom panels not described, never tested with a screen reader |
| **UI/UX overall** | **~55%** | **~70–110** (overlaps the feature areas) | estimated |

The overall figure is lower than the rows because what users notice first (small differences across many
tools, never checked side by side) isn't captured row by row.

## Pen, Curvature and anchor tools

| Behaviour (Illustrator, public docs) | Status | Notes |
|---|---|---|
| Click: corner anchor; drag: smooth anchor with symmetric handles | done | `pen.rs`; the drag's preview follows every pointer position, back into the window after leaving it and in the release's frame, and the release commits where the pointer let go (#834) |
| A press that moves under 3 screen pixels is a click (a corner without handles) | done | #1663 |
| Shift constrains segments and handles to 45° | done | Illustrator's Constrain Angle preference is honoured (`constrainAngle`): segments, the Pen's handles, Shift moves with the selection tools, Free Transform, slices, Reshape and Measure (#394) |
| Alt mid-drag splits the handles (only the outgoing follows) | done | keeps the curve already shaped |
| Space mid-drag repositions the anchor, handles and all | done | |
| Click the first anchor to close; the drag shapes the closing curve | done | |
| Click the last anchor to retract its outgoing handle; drag from it to pull a new one | done | |
| Alt over a handle, anchor or segment acts as the Anchor Point tool | done | |
| Cmd/Ctrl lends the last-used selection tool | done | the path being drawn continues afterwards |
| Click an open path's end to continue it; click another path's end to join | done | #776 |
| Rubber Band preview | done | Preferences option |
| Auto Add/Delete on a selected path; Shift or the preference disables it | done | |
| Pen cursors (new path ×, continue /, close o, add +, delete −, convert ^) | done | the nib is a fountain-pen nib, tip on the hotspot, with the slit widening into the breather hole and the heel clipped square to its axis; bitmap tests at 1× and 2× pin the nib and the new-path asterisk, and in-app screenshots compare the Pen with Direct Selection |
| Node-editing cursors: Direct Selection's white arrow with a hollow or filled square over an anchor (path unselected or selected); the Reshape Segment cursor over a curved segment (Direct Selection, Anchor Point, the Pen with Alt); the Anchor Point caret for the Pen with Alt; Group Selection's plus (also Direct Selection with Alt); a filled arrow while dragging that turns hollow on a snap; a double arrowhead for an Alt-drag copy | partial | as the public manual describes them, drawn in code; no cue over handles (none documented) |
| Anchor Point and Curvature tools' own cursors | done | the caret on the anchor it converts; the pen nib with a curve badge |
| Free Transform's Cmd/Ctrl side-shear cursor | done | two arrows passing along the side (ShearH, ShearV), instead of the resize arrow across it |
| Cursors of their own for Lasso, Magic Wand, the freehand tools, Shaper, Live Paint Bucket, Measure, Print Tiling, the symbolism tools, vertical type, Scissors (over a segment and away), Mesh and Puppet Warp | partial | each tool returns its own cursor and Use Precise Cursors turns them into the crosshair; until they're drawn they borrow the crosshair, the I-beam, the Pen's add cursor or not-allowed |
| Busy cursor during long operations | done | Background open, save and export show the system progress cursor (gap G1) |
| Each anchor snaps to Smart Guides and construction angles | done | `DrawSnap` |
| Curvature tool: click/drag points, double-click or Alt-click for a corner, temporary Direct Selection, point conversion | partial | circular point markers without direction handles or duplicate markers, Anchor Size and Hide Edges regression-tested (#1236, in part); curve geometry: each stretch of three points follows the circle through them (handles along its tangent, chord/(3·cos²(α/2)) long, shorter in a turn tighter than 30°), open ends curving in, the same rule for the rubber band, dragged, inserted and converted points (#1236, in part); neighbours of a point dragged on an existing path keep their tangents, and modifiers requested in #975 remain |
| Blend spine Anchor Point conversion and handle editing (#2544) | implemented | symmetric handles, independent drag/retraction, visible-handle policy and selection/stacking covered by tool and engine regressions; Windows native canvas checks and screenshots reviewed in PR #2643 |
| Add/Delete/Anchor Point tools; Anchor Point tool drags segments to reshape | done | #2267: a click on a handle end retracts that handle and leaves the anchor a corner with the other one; Shift held at the press on an anchor moves it, handles and all (Shift kept down: at 45° steps) instead of pulling out handles |
| Pencil: Fidelity, Keep selected, Alt to Smooth, edit selected paths, close by proximity | done | Alt key toggles to Smooth Tool (#2001); Shift held at the press draws a straight line at 45° steps from the Constrain Angle, and Alt held at the press (with Alt key toggles to Smooth Tool off) a straight line at any angle (#3799), with a straight-line cursor while Shift or Alt is held before the press, and a path-continuation cursor over an end of a selected open path, where the line continues the path; Edit Selected Paths continues a selected path from an end and, from a stroke that starts and ends on a selected path, redraws the section between (the shorter way round a closed path, #2850); with Keep selected (Pencil and Paintbrush, on by default) off, the path a stroke draws or continues is not selected afterwards; a filled Pencil path keeps a stroke of None (#1804, part of #1679); while a freehand stroke is drawn back to within Close Paths When Ends Are Within of its start, the Pencil and the Paintbrush show the closing cursor, and a release there closes the path |
| Join tool scrubbing, Average, Join | done | Join merges the direct-selected anchors lying on one another along a path, as after Average, into one (#1533) |
| Average and Join in one step, with a corner or smooth choice for the joined anchor | partial | `path.join` accepts `to: corner|smooth` on the merged anchor; Average and Join are still separate commands; [gap 1](gaps.md) |
| Smooth tool smooths only the stretch of path under the drag | done | #1512 |
| Drawing modes apply to pastes: Draw Inside takes Paste, Paste in Place and Paste on All Artboards inside the object; Draw Behind puts Paste in Place and Paste on All Artboards behind the selection; Shift+D cycles past Draw Behind with any selection ([drawing modes](https://helpx.adobe.com/illustrator/desktop/draw-shapes-and-paths/learn-drawing-basics/drawing-modes-overview.html)) | done | #3705; `pasting_in_draw_inside_puts_the_copies_inside_the_object`, `paste_in_place_in_draw_behind_goes_behind_the_selection`, `shift_d_goes_past_draw_behind_whatever_is_selected` |
| Simplify (slider, auto, corner angle threshold), of a whole path or of the part of it selected with Direct Selection ([manually simplify paths](https://helpx.adobe.com/illustrator/desktop/draw-shapes-and-paths/modify-paths/manually-simplify-paths.html)) | done | Tolerance, default-on Preview and original/new point counts (#1272); Corner Point Angle Threshold, Convert to Straight Lines and Show Original Path; the slider under the selection with Auto-Simplify and More Options, opening at the Auto-Simplify level for the selection with that result previewed, and the dialog's option to open it directly with its last settings. With anchors or segments direct-selected, the slider, Auto-Simplify and the dialog refit only each run of selected segments between its end anchors, a closed path's run across its start point included; the rest of the path stays exactly as it was (`tests_simplify_part.rs`, `simplify_slider_and_dialog_simplify_only_the_direct_selected_part_of_a_path`) |
| Draw Inside in isolation mode: art drawn or pasted inside the isolated object takes isolation mode into its clip group, clickable and undimmed ([isolation mode](https://helpx.adobe.com/illustrator/desktop/manage-objects/select-objects/isolate-objects.html), [drawing modes](https://helpx.adobe.com/illustrator/desktop/draw-shapes-and-paths/learn-drawing-basics/drawing-modes-overview.html)) | done | #3705; `drawing_inside_an_isolated_object_keeps_the_new_art_in_isolation_mode` |
| Simplify (slider, auto, corner angle threshold) | partial | Tolerance, default-on Preview and original/new point counts (#1272); Corner Point Angle Threshold, Convert to Straight Lines and Show Original Path; the slider under the selection with Auto-Simplify and More Options, and the dialog's option to open it directly with its last settings; the slider opens at 1 pt rather than at the Auto-Simplify level, and simplifying only a Direct Selection part of a path remains missing |
| Simplify (slider, auto, corner angle threshold) | partial | Tolerance, default-on Preview and original/new point counts (#1272); Corner Point Angle Threshold, Convert to Straight Lines and Show Original Path; the slider under the selection with Auto-Simplify and More Options, opening at the Auto-Simplify level for the selection with that result previewed, and the dialog's option to open it directly with its last settings; simplifying only a Direct Selection part of a path remains missing |

## Direct Selection, anchors and handles

| Behaviour | Status | Notes |
|---|---|---|
| Click/Shift-click anchors; marquee toggles with Shift | done | #483 |
| Align selected anchors to their bounds or the artboard; repeating an alignment that moves nothing keeps Undo, Redo and saved state | done | #1180 follow-up; engine regressions include rounding-only moves and unchanged handles; #1641 (fixes #1636): Align and Distribute act on the anchors selected with Direct Selection, the points of one path aligning to each other and Distribute spacing them |
| Align selected anchors to a key anchor | missing | #1636 |
| A ruler guide as the key object: a click on a guide selected with objects (or Align to Key Object with one object and a guide) makes it the key, drawn thicker; Align lines the objects (or selected anchors) up on it and the guide stays put | done | #2143; `select.key {guide}`; a vertical guide sets x only, a horizontal one y |
| Distribute Objects and Distribute Spacing with a spacing from a key guide: the guide stays put and the other items are spaced the Distribute Spacing value from it, as from a key object with no size | done | Key guide from #3165; a vertical guide spaces the horizontal options, a horizontal one the vertical ones, and a guide along the axis leaves everything |
| Distribute ruler guides: the Distribute Objects and Distribute Spacing buttons space the selected guides across the axis with the selected objects, as items with no size; two or more guides selected alone are enough | done | #1597 in part; guides alone: select them, choose Align to Artboard and click Horizontal (Vertical) Distribute Center for equal columns (rows) ([public forum answer, 2017](https://community.adobe.com/questions-652/illustrator-equal-guides-822924)); with objects: a guide on each artboard edge, included in the selection, keeps the outermost lines there ([public forum answer, 2025](https://www.signs101.com/threads/is-there-a-way-to-distribute-shapes-like-this-in-illustrator.179556/)); a guide has no width, so the three options of an axis are the same for it; guides along the axis stay |
| Ruler guides align to a key object: a click on an object selected with guides makes it the key (one object is enough); Align moves the selected guides to its edge or centre (vertical guides to left, centre or right, horizontal ones to top, centre or bottom) and the key stays put | done | #2143; aligning to the selection or the artboard leaves guides alone |
| A marquee selects the objects whose paths it touches (anchors or segments): drawn inside a filled shape, over a background, it leaves that shape (Preview and Outline) | done | #1103 |
| Click a segment independently of its endpoint anchors; only its direction handles show | done | #1278: hollow endpoint squares, selected edge emphasized; headless egui checked |
| Drag a curved segment to bend it, a straight one to move its anchors | done | |
| Drag by an anchor of a path selected whole moves the whole path; Alt-drag by an anchor or a segment copies it, Alt toggles the copy during the drag, and the copy snaps by the anchor pressed, onto the original too | done | #1104 |
| Handles shown are the handles that drag; tolerance from Selection & Anchor Display | done | #494 |
| Drag a handle with Shift (45° about its anchor), Alt (move one handle alone) | done | handles snap to smart guides; handle and anchor drags no longer stall when the pointer leaves and re-enters the window, and a move in the release's frame lands (#834) |
| Show handles when multiple anchors are selected; Highlight anchors on mouse over; anchor and handle size | done | Preferences › Selection & Anchor Display |
| Convert selected anchors to corner/smooth from the Control bar and Properties | done | |
| Cut path at selected anchors, Remove Anchor Points, Delete removes segments | done | Open-path anchor Delete removes adjoining segments without bridging the gap; targeted core and full-frame Delete/Backspace regressions pass for survivor selection and Undo/Redo. Explicit Remove Anchor Points keeps its reconnect/refit behavior; #1278: segment Delete preserves neighboring edges and endpoint anchors; a second Delete removes the remainder; explicit anchor Delete retains its behavior; #1674: Delete removes every selected anchor of a closed path and objects selected whole along with them, and keeps the remainder selected; #2241: Remove Anchor Points then selects the next anchor along the path (the one before at an open end), so it can be run again without picking each anchor |
| Direct Selection and Group Selection pick only inside the isolated object in isolation mode | done | #1666 |
| Group Selection picks one path of a compound path (the outer shape or a hole) and moves it on its own; a second click adds the compound path | done | #2441; Shift-click on a selected member deselects it on release |
| Direct Selection, Group Selection and the Selection tool take a clipping path (a clip group's, or a layer's clipping mask) on its outline, within the selection tolerance, even where the art it clips lies over its edge | done | `HitOptions::clip_outlines`; Select Behind lists the same; the eyedropper and the tools that act on what is drawn take the art |
| Escape cancels a Direct Selection drag | done | #1803 (#1743) |
| Live Corners widgets (radius readout, Alt-click cycles kind, double-click the Corners dialog) | done | a Direct Selection click on a widget selects its corner (#1239); a press-and-drag on a widget of a path selected whole still rounds every corner (#908), and adding widgets to the selection by Shift-click (#1232) is missing; the bounding box hides while a widget is dragged with the Selection tool (#1229); a round corner next to a curved side curves smoothly instead of sticking out of the shape (#1512); a corner rounds until its cut reaches its neighbours' cuts, so one corner of a square alone reaches the whole side (#2332) |
| Corners between curved sides (#1144), Relative rounding | done | #1144 closed; Relative rounding in the Corners dialog (Relative / Absolute, radius as a percentage of each corner's maximum, kept as the shape is resized; `object.setLiveShape` `relativeRadii`, #3091, `tests_livecorners::relative_corner_rounding_keeps_fraction_on_resize`) |
| Pie widgets on a live ellipse: drag one to set Pie Start or End Angle, double-click one to make the ellipse whole | done | Selection tool, with the bounding box showing |
| Isolation mode by double-click (groups and single objects), breadcrumbs, Esc to exit | done | Single paths, compound paths, images, meshes and placed files isolate too ([helpx: Isolate objects](https://helpx.adobe.com/africa/illustrator/desktop/manage-objects/select-objects/isolate-objects.html)) |
| "Magnetic" moves of selected anchors with a modifier (#955 request) | missing | not an Illustrator feature as documented; under discussion |
| Coinciding anchors of overlapping objects: the selected object's anchor is taken, also under another object | done | #1508 |

## Smart guides and snapping

| Behaviour | Status | Notes |
|---|---|---|
| Snap to anchors, centres, edges, artboards, bleed and ruler guides with magenta lines | done | `guides.rs` kinds: Anchor, Center, Edge, Artboard, Bleed, Guide |
| Anchor/Path labels ("anchor", "center", "path", "align") | done | |
| "intersect" label and snapping to path intersections | partial | #1658 (part of #1602): a point drawn with the Pen, the shape tools or the other drawing tools and an anchor dragged with Direct Selection snap to where paths and ruler guides cross; a moved selection's own anchors and the Width tool don't |
| Construction guides at preference angles while drawing | done | #506 |
| Line extension: the dragged end of a straight segment snaps to the segment's own direction at any angle, past its other end or short of it, with the guide along it | done | #2120; Direct Selection and Curvature, one anchor dragged (with several, the move has no single direction); nearer the horizontal or vertical through the other end, the point lines up with it ("align"), so a nearly level segment can still be straightened |
| Measurement labels while drawing, moving and resizing | partial | sizes and offsets; angle and distance readouts while drawing missing |
| Spacing guides (equal distances) | partial | for moves and bounding-box resizes (#394); not while drawing |
| Distance Guides: Alt/Option over another object, or an empty part of an artboard, shows the selection's distances to it (Smart Guides › Invoke Distance Guides) | done | Selection tool hover with Smart Guides on: the gap between facing edges, else the offsets of matching edges, to another object; the left, right, top and bottom distances to the artboard; labelled in the General unit. The selection is measured as a whole. Nothing while moving or nudging, which the public pages don't describe. [Use Distance Guides](https://helpx.adobe.com/illustrator/desktop/measure-and-align/grids-and-guides/use-distance-guides-for-accurate-placement.html), [Smart Guides options](https://helpx.adobe.com/illustrator/desktop/measure-and-align/grids-and-guides/smart-guides-options.html) |
| Snapping tolerance preference | done | |
| Snap to Point, Snap to Grid, Snap to Pixel; Pixel Preview | done | #1840: with Snap to Grid on, anchors and objects snap to the grid even when Snap to Pixel is also ticked (#1816 waits for the reporter) |
| Snap to Glyph | done | View › Snap to Glyph (`view.snapToGlyph`); new type and drawing snap to live-text baselines and glyph bounds when Smart Guides are on (#3124) |
| Tangent and perpendicular snapping for line endpoints (new in 30.0) | partial | #2530: tangent snapping to curves and perpendicular snapping to straight segments for drawn lines; saved Preferences › Smart Guides › Geometric Guides controls, on by default. Other tool/drag cases remain. [Public source](https://helpx.adobe.com/illustrator/desktop/measure-and-align/plot-and-measure/snap-to-perpendicular.html) |
| Snapping panel/popover consolidating the snap options (30.0) | partial | Control bar › Snapping provides quick access through existing view commands and prefs.set; no View › Snapping submenu. Floating-panel parity remains unverified. |
| Customisable rotation snap angle (#973) | missing | Illustrator: Constrain Angle and construction angles only |
| Ruler guides: drag out, select, move, delete, lock, Make/Release Guides | done | #414, #451; marquee selects unlocked ruler guides |
| Escape cancels an existing ruler-guide press, move or copy | done | Selection and Direct/Group Selection; app-input regressions cover preselected guides, document/interaction-start selection, Undo/Redo and saved state, later on-canvas motion/release and temporary borrowing from idle Pen. All eight checks and the Direct Selection Escape control passed in current full CI. [Validation evidence](target-app-parity.md#evidence-what-ships-today-by-area) |
| Ruler origin: corner drag with preview, double-click reset to the active artboard, relative X/Y fields, native save/reopen | done | #1212; `document.setRulerOrigin`, `document.resetRulerOrigin`; geometry and command coordinates stay absolute |
| Ruler guides as Layers rows: shown, hidden, locked and deleted with their layer; rows dragged to another layer | done | `guide.setLayer`; guides from older files go on a layer |
| Objects made into guides: drawn in the guide colour and style of Preferences; Lock Guides locks them too (no select, move or delete); a dragged selection snaps to ruler guides with Smart Guides on, by the nearest point of the path when pressed on its stroke or beside its edge, so the path lands exactly on the guide | done | #1534 (#1543, #1583); #3701 |
| A selected ruler guide's X or Y in the Transform panel; typing a value moves it | not scored | #1659 (part of #1597); several guides move together; not in Properties or the Control bar. Not established by the documentation we could find: the help pages on guides, rulers and the Transform, Control and Properties panels (read again on 2026-10-14) don't describe it, and the one public statement is a user's [community answer](https://community.adobe.com/questions-652/adding-a-new-custom-guide-785586), which is not documentation. Kept as a convenience pending the owner's decision; no parity claim |
| Alt swaps a ruler guide's orientation while dragging | missing | Alt-drag copies the selected guides ([Use rulers](https://helpx.adobe.com/illustrator/desktop/measure-and-align/grids-and-guides/use-rulers.html)); `guide.move` `{copy: true}` |
| Alt swaps a new ruler guide's orientation while dragging | done | Both rulers; preview, snapping and final placement follow the current modifier ([public shortcut reference](https://helpx.adobe.com/illustrator/using/default-keyboard-shortcuts.html)). |
| Global/artboard ruler mode switch (View › Rulers › Change to Global / Artboard Rulers, Cmd+Alt+R) | done | `view.globalRulers`; a new document has artboard rulers, counting from the active artboard's top-left corner or the zero point set for it (kept relative to the artboard); global rulers keep one document zero point. Rulers, Transform, Properties, Control and Info X/Y, status-bar pointer X/Y, guide readouts and Shift ruler-tick snapping follow the mode; saved with the document, older files open with global rulers. Behaviour from [About rulers](https://helpx.adobe.com/illustrator/desktop/measure-and-align/grids-and-guides/about-rulers.html) and [Use rulers](https://helpx.adobe.com/illustrator/desktop/measure-and-align/grids-and-guides/use-rulers.html) |
| Video rulers (View › Rulers › Show Video Rulers) | done | #2849; `view.videoRulers`: the rulers show and count the active artboard in video pixels from its top-left corner, whatever the ruler origin, a pixel as many points wide as the artboard's Video Ruler Pixel Aspect Ratio and one point tall ([public source](https://helpx.adobe.com/illustrator/desktop/create-manage-artboards/organize-manage-artboards/set-video-display-options-for-artboards.html)); the item then reads Hide Video Rulers, which returns them to the General unit. Shift ruler-tick snapping of new guides keeps the General unit. Tests `video_rulers_count_the_artboard_in_pixels_of_its_aspect_ratio`, `show_video_rulers_counts_the_active_artboard_in_pixels` |

## Transform handles, bounding box, Free Transform

| Behaviour | Status | Notes |
|---|---|---|
| Moving a rounded rectangle with Craquelure at high DPI | partial | Distance searches optimized; moved-viewport pixels unchanged for three texture effects; `texture_drag_perf` probe available. Idle Windows mouse-drag QA remains (#1257) |
| Bounding-box handles: Shift proportional, Alt from centre, rotate outside the corners | done | |
| Rotated objects keep a rotated bounding box and their angle; Reset Bounding Box | done | |
| Free Transform, Scale and Reflect along a rotated box | missing | |
| Free Transform widget: constrain, free distort, perspective distort; the key-held variants | done | |
| Transform tools: click sets the reference point (snaps to anchors/centres), Alt-click opens the dialog, Alt-release copies | done | with anchors selected with Direct Selection, the tools, the dialogs and Move transform those anchors alone, about the centre of their box by default (#1408; the default point and whole-object copies are our choices) |
| Move, Rotate, Scale, Reflect and Shear dialogs: live Preview; Up/Down step angles and percentages by 1, Shift by 10 | done | #1412; Preview starts off (our choice: the public documentation gives no default), Move's as it was last confirmed; OK is one undo step and Transform Again repeats it |
| Transform Again, Transform Each, Scale Strokes & Effects, Scale Corners | done | |
| Move opens on the offset of the last move (a drag of the selection or Move) and with the Copy, Transform Objects and Preview it was last confirmed with, so Enter, Enter repeats a move or a copy | done | the public source: [an Adobe Community thread](https://community.adobe.com/questions-652/move-tool-771221) (the dialog shows the last move, drags included); that the checkboxes are kept, Preview among them, and that a nudge with the arrow keys is not a move here are our choices; Transform Patterns is General › Transform Pattern Tiles, which Move's OK sets; the offset is the session's (not kept between launches) |
| Transform Objects beside Transform Patterns in the Move, Rotate, Scale, Reflect and Shear dialogs and Transform Each; Reflect across an angle | done | #1448, #1460; off, only the pattern tiles transform; the checkboxes are always enabled (our choice); the link that keeps Transform Each's two scales equal is missing (not documented) |
| Split Into Grid dialog: number, height or width, gutter and total for rows and columns, Add Guides, Preview | done | #1462; opens at the selection's size; several paths become one grid painted as the topmost one; a typed height or width keeps the gutter and changes the total (our choice) |
| Reference point locator (nine squares) in Control bar and Transform panel | done | |
| Transform panel, Properties and Control bar X/Y/W/H of the anchor points selected with Direct Selection | done | #1839 (fixes #1824): they show and edit the selected points' bounds; a whole-object selection is unchanged |

## Nudging, numeric entry and precision

| Behaviour | Status | Notes |
|---|---|---|
| Arrow keys nudge by Keyboard Increment (0.001–1296 pt); Shift ×10; Alt copies | done | |
| Fields commit what was typed on a click away or a move to another field, as Enter does; Escape discards it | done | #1787 |
| Number fields: Up/Down step, Shift ×10, Cmd/Ctrl ÷10, wheel over a focused field; steps, the wheel and label scrubs stop at the field's bounds (a stroke weight at 0, a miter limit at 1) | done | Shift+arrow in the Stroke weight field doesn't step by 10 (#991); bounds #3370 |
| Arithmetic in fields (`+ - * /`, and `%` of the value before it: `3cm*50%`, `50pt+25%`) and unit suffixes (`3 mm`, `1in`, `2p6`) | done | `Unit::parse`; mixed units in one expression not verified |
| Numeric fields: focus click selects the whole value; dragging selects from the pressed character | partial | #1326; select-all waits for click release; px/pt/plain/optional fields, both directions and replacement text tested; native API input, 17 px partial replacement, Undo/Redo, saved-file round trip and screenshots verified; Tab, double-click, arrows and wheel preserved; short movements still classified as clicks can select all and remain open |
| Units: pt, pc, in, mm, cm, px, ft-in, m, yd, ft; General/Stroke/Type units | done | units preference reverting to points on SVG output reported (#864) |
| Field precision: 3 decimals for steps; values kept as f64 | done | |
| Scrubby labels (beyond Illustrator) | done | #400 |
| 3D Lighting Color picker, preview, reset dot and Reset all | initial | One to ten opaque RGB lights for Revolve, Extrude & Bevel, Inflate and Rotate; select, add or delete a light and edit its settings or gizmo; Standard, Diffuse, Top Left and Right select original rigs, with Custom for manual changes; Cancel restores all lights |
| Dialogs open with the first field focused and selected; Enter confirms | done | on every open, not only the first (#1367); a first field that starts greyed out takes the focus once enabled (#1395) |
| Value fields: selected text readable in every brightness; a focused field follows its value when a slider or preview changes it | partial | #1490 (part of #1465: the wheel over a hovered, unfocused field still scrolls the panel), #1457 |
| Sliders in dialogs and popups (Save for Web, raster export options, Preferences, the opacity popup) show their rail, filled up to the handle | done | #1483 |
| Rectangular Grid and Polar Grid click dialogs: size, divider counts with a skew slider each, Use Outside Rectangle As Frame / Create Compound Path From Ellipses, Fill Grid; kept as the tool's options, used by drags too | done | #1377. The options are the documented ones; how far a skew percentage moves the dividers and which way the radial skew turns are our own choice (the public documentation gives neither). The reference point locator is in the dialog, and a double-click on the tool opens it as the tool's Tool Options (next row) |
| Double-click the Line Segment, Arc, Rectangular Grid or Polar Grid tool: its Tool Options ([Rectangular Grid tool](https://helpx.adobe.com/illustrator/using/tool-techniques/rectangular-grid-tool.html) and [Arc tool](https://helpx.adobe.com/illustrator/using/tool-techniques/arc-tool.html) step 3, [Tools](https://helpx.adobe.com/illustrator/using/tools.html)) | done | #2691. The tool's click dialog without a place to draw: Line Segment's Length, Angle and Fill Line; Arc's lengths, Reference Point, Type, Base Along, Slope and Fill Arc; the grids' Reference Point, Default Size, dividers, skews and switches. OK draws nothing and sets what the next drag draws with (Fill Line, the arc's type, base, slope and fill, the grids' dividers, skews and switches) and what a click's dialog opens at (with the Length, Angle and Default Size); Cancel changes nothing (`double_clicking_the_line_arc_and_grid_tools_opens_their_tool_options`) |

## Modifier keys and temporary tools

| Behaviour | Status | Notes |
|---|---|---|
| Space: Hand; Cmd+Space / Cmd+Alt+Space: zoom in/out; middle button pans | done | |
| Document window scroll bars along the right and bottom edges (Normal and Full Screen Mode with Menu Bar; none in Full Screen and Presentation Mode): a thumb as long as the view against the art and the artboards; dragging it scrolls, a click on the track pages, the wheel over a bar scrolls along it; in a rotated view they follow the screen's axes | partial | #2322. The span is the art and the artboards with half a window around them, not Illustrator's 227 in canvas: VectorCraft has no bounded drawing area (#2284). A track click pages rather than jumping to the click; the page size and the rotated-view behaviour are our own choice (the public documentation gives neither) |
| Cmd/Ctrl: last-used selection tool; Cmd+Alt: Group Selection inside Direct Selection | done | Cmd+Alt lends Group Selection only while Direct Selection is active (Cmd borrow from the Pen keeps Direct Selection so Alt stays copy/subtract); engine pointer regressions |
| Eyedropper styles highlighted characters only and preserves ranges across Type switches | done | Font/paint, Shift color-only, Unicode, Undo/Redo and stale-range regression tests; enabled non-neutral source or target object transparency is refused atomically; disable Transparency to sample supported attributes |
| Eyedropper reads visible sources while their layer/object is locked or outside isolation | done | Ordinary/Shift/Shift+Alt and gradient-stop reads preserve the source and selection; Alt reverse remains editable-only; hidden art stays excluded (`eyedropper_sources.rs`, #1251) |
| Add a visible stroke after drawing with inherited None/zero-width paint | done | Paint and Swap Fill/Stroke restore the existing 1 pt default; positive widths and painted zero widths stay as set; Undo/Redo and targeted appearance items tested (`stroke_visibility.rs`, #1253) |
| Double-click a stroked segment to create a width point in Width Point Edit | done | Finite-arithmetic follow-up: engine regressions and full CI passed; [Validation evidence](target-app-parity.md#evidence-what-ships-today-by-area). Command and pointer regressions cover opening, sampled widths, adjoining adjustment on/off through checkbox/OK, compound paths, Cancel/Delete and Undo/Redo; dialog layout reviewed. [Validation evidence](target-app-parity.md#evidence-what-ships-today-by-area) |
| Shift-double-click a width point enables adjoining adjustment in Width Point Edit | done | Tool dispatch, plain double-click control, neighboring widths, Undo/Redo and Cancel in `dialogs/width_point.rs` |
| Alt-drag duplicates (objects, artboards with their art) | done | |
| Document Setup → Edit Artboards applies valid settings, closes the dialog and activates the Artboard tool | done | Pointer-click tests cover successful handoff, invalid settings, OK, Cancel and Undo in `dialogs/document_setup.rs`. |
| Escape leaves Artboard editing for Selection; during a drag it first cancels the gesture | done | Headless egui key events cover idle Escape, active-drag cancellation and a subsequent Escape in `tests_distortkeys.rs`. |
| Escape while typing ends the edit, keeps the type selected and switches to the Selection tool | done | #1802 (#1686, #1656) |
| Selection tool: selected area type and type on a path show an in port and an out port (empty, an arrow where threaded, the red "+" of overflow); a click on either loads the text cursor; a click on a closed path, on empty canvas or a drag threads the text there, after the clicked frame from its out port, before it from its in port; Escape or another tool drops it | partial | #2572 (`text.thread.add`, `before` for the in port); the out port sits on the frame's right side above the bottom-right corner handle and the in port on its left side below the top-left one, so the handles can still be grabbed; a double-click on a port doesn't break the thread yet; the loaded cursor borrows the I-beam |
| Shift-click selects several artboards (Artboard tool, Artboards panel); they move, Alt-copy and delete together | done | `artboard.move`/`artboard.delete` `indices` (#1139); panel menu, trash and Properties share selected-set deletion; all-selected retains artboard 1 and a single board cannot be deleted, with pointer-event and Undo/Redo coverage |
| Artboard Delete/Backspace delete selected boards while protecting artwork, the last board and active drags | done | Headless shortcut regressions in `tests_distortkeys.rs` cover retaining the original first board when all are selected, single-board protection, artwork, saved state and Undo/Redo; the navigator follows the tool’s surviving board through repeated deletion and history changes. |
| Artboard navigator synchronizes the active tool selection before deletion | done | Actual status-bar Next → Delete/Backspace and Next → Properties Delete regressions in `chrome.rs` cover surviving board records, preserved artwork and Undo/Redo; other tools keep object selection and history. **Tab-switch follow-up:** document-tab changes align the Artboard tool with the displayed board. Actual A → B → A tab clicks followed by Properties Delete and controls for same-tab multiselection, Move/Scale Artwork options and other tools passed, along with historical full CI on `a0671777`. [Validation evidence](target-app-parity.md#evidence-what-ships-today-by-area). **Clean-tab close:** with the Artboard tool active, closing a clean active tab restores the surviving document's displayed board before Properties Delete. |
| Artboard history prunes removed indices while preserving valid multiselection and tool options | tested | Headless pointer/key tests in `panels/properties.rs` cover panel New → Undo → Delete/Backspace/Properties, Duplicate → Undo → Delete, partial pruning and valid multiselection. They check surviving board records, artwork, object selection, Move/Scale Artwork options, saved state and Undo/Redo. Full workspace CI passes. |
| Click an object with the Artboard tool to create an artboard fitted to its visible bounds | done | Topmost editable object or group; drag and modified clicks retain their meaning; one undo step (`artboard_click.rs`, #1227) |
| Artboard Options: X and Y beside a reference point locator, counted from the ruler origin; a typed X or Y moves the artboard's reference point there, a new Width or Height keeps it in place | done | #1876; as the Transform panel's and the Control bar's artboard X and Y, with the locator they and Properties share; the shape dialogs' X and Y (the click position) stay hidden. Pointer and key tests in `dialogs/tests.rs` |
| Artboard Options shares existing presets/orientation and drafts artwork or selected-art fitting until OK | done | Fit changes only the active artboard and preserves artwork positions; Cancel leaves the document unchanged; newer multi-artboard behavior retained; fitting after draft resizing uses all drawn document artwork and document/revision-scoped bounds; selected art may span boards |
| Artboard Options › Display: Show Center Mark and Show Cross Hairs ([public source](https://helpx.adobe.com/illustrator/desktop/create-manage-artboards/organize-manage-artboards/modify-display-settings-of-artboards.html)) | done | Per-artboard checkboxes (`artboard.setProps`/`artboard.new` `showCenterMark`, `showCrossHairs`, one undo step); the canvas draws a cross 12 points across on screen at the center and lines from the middle of each side through it, 1 point wide on screen at any zoom and turned with the view; exports and print leave them out; tests `an_artboard_shows_its_center_mark_and_cross_hairs`, `artboard_options_show_center_mark_and_cross_hairs` |
| Artboard Options › Display: Show Video Safe Areas and Video Ruler Pixel Aspect Ratio ([public source](https://helpx.adobe.com/illustrator/desktop/create-manage-artboards/organize-manage-artboards/set-video-display-options-for-artboards.html)) | done | #2849; per artboard (`artboard.setProps`/`artboard.new` `showVideoSafeAreas`, `videoRulerPixelAspectRatio` 0.1–10, default 1; one undo step; native format). The page gives no safe-area numbers: our choice is [EBU R 95](https://tech.ebu.ch/docs/r/r095.pdf)'s, the action-safe area 3.5% and the title-safe area 5% in from each edge, outlined 1 point wide on the canvas and turned with the view; exports and print leave them out. Tests `an_artboard_shows_its_video_safe_areas`, `artboard_options_show_video_display_options`, `artboard_video_display_options_*` |
| Shape tools: Shift constrains, Alt from centre, Space moves while dragging, arrows change sides/points/rows | done | |
| Polygon/Star exact-size placement remembers the chosen sides/points for later drags and dialog openings; Cancel preserves the previous count | done | Session tool options; existing dialog-opening gestures retained; regression in `toolbar.rs` |
| Tool double-click opens its options | done | |
| Per-platform mapping (Cmd ↔ Ctrl, Option ↔ Alt) | done | |
| Eyedropper cursor; Use Precise Cursors | partial | original pipette at the sampling tip, distinct from the precise crosshair at five DPI scales (#1269); OS cursor display and Caps Lock unverified; a Precise Cursors command toggles the preference and can take a shortcut (#1663); cursors are painted into the window on Wayland (#1652, not run on Linux) |
| Every tool's modifiers checked against the shortcut reference | missing | this is the fidelity pass |

## Keyboard shortcuts

Cmd/Ctrl+/ (`appearance.addFill`) and Cmd/Ctrl+Alt+/ (`appearance.addStroke`) add a new fill and a new stroke to the
selected objects, as the Appearance panel's Add New Fill and Add New Stroke buttons do.

~110 distinct modified shortcuts on menu commands plus single-key tool shortcuts for the default toolbar
(`crates/tools/src/catalog.rs`), a shortcut editor with chords, and saved sets. Cmd/Ctrl+Alt+L (`ui.newLayer`) opens Layer Options for a new layer, as an Alt-click on the Layers panel's New Layer button does, and Cmd/Ctrl+L still adds a layer without the dialog. F8 (`ui.symbolOptions`) opens Symbol Options for a new symbol made from the selection, as the Symbols panel's New Symbol button does. Not yet diffed item by item
against Illustrator's published default shortcut list; estimated ~70% of it. Remaining: the panel shortcuts,
the type shortcuts (tracking/kerning/leading/baseline steps with Alt/Shift), and selection shortcuts in the
Layers panel. The shortcut editor also lists the commands that are in no menu but run without parameters (Edit › Duplicate,
panel-menu commands such as Break Link to Symbol), so they can be given shortcuts (#1539). It lists each button of the
Align panel's Align Objects, Distribute Objects and Distribute Spacing rows under Align, with no default shortcut, and a
shortcut set there runs what a click on the button runs, with the panel's Align To choice and Distribute Spacing value. The [ and ] keys reach the Blob Brush, the Eraser and the Symbolism tools in the app (#1734, part of #1661).

Cmd/Ctrl+Alt+N (`file.newWithoutDialog`, New Without Dialog in the shortcut editor) makes a document with the
settings the New Document dialog opens on (the first Recent entry, or the Print category's Letter while Recent is
empty), without opening the dialog.

## Panels, Control bar, Properties, context menus

| Interaction | Status | Evidence |
|---|---|---|
| Selection artwork drag hovers a visible document tab to activate it, then drops editable artwork on its canvas | done | `tests_document_drag`, `edit.dropSelection`, lossless resource and one-step Undo regressions; overflow and cross-window drag remain separate |

| Behaviour | Status | Notes |
|---|---|---|
| Effect > Warp Options: style previews, orientation choices and percent controls | done | #1760; focused input, mixed-style and history checks passed; original captures reviewed. No geometry or optional Properties changes. |
| Command search: Up and Down move through the results, Enter runs the highlighted one | done | #1798 (#1757) |
| Pathfinder panel: a group selected alone counts its nested shapes, so an expanded stroked object keeps the Shape Modes enabled | done | #1642 (#1635) |
| Selected Mirror Repeat shows a prominent axis with move and rotation handles; bounding-box handles take priority over the axis line | done | #2877, `tools/src/mirror.rs`; [public behavior reference](https://helpx.adobe.com/illustrator/desktop/paint-and-fill/create-and-edit-patterns/set-mirror-repeat-options.html) |
| Object › Expand on radial, grid and mirror repeats makes groups of their instances | done | #1765 (#1756) |
| Select › Same matches type on its characters' fill and stroke | done | #1746 (#1737) |
| ~40 panels with ≡ menus; dock, icon column, collapse, tabs | done | 51 panel modules incl. tests |
| Panels float inside the window, stack, dock back | done | #495 |
| Floating groups stack into sets; sets dock as extra dock columns (a multi-column dock); groups drop anywhere in a set or column, or onto its tabs; a column taller than the window scrolls | done | `window.panel.float {below, above}`, `window.panel.dock {column}` |
| Panels dragged onto a dock group's tabs join it; onto its upper or lower half they make a new group above or below it (a top/bottom split, resizable); onto the icon column they become icons, the tabbed group's panels too; a dock tab's context menu closes it (Window menu reopens it) | done | `window.panel.dock {anchor, zone}`, `window.panel.dock {icon: true}`, `window.panel.close` |
| A double-click on a dock group's tab collapses the group to its tabs (a group below it moves up into the room) and a second one expands it; a click on another of its tabs expands it showing that panel ([helpx: Workspace basics](https://helpx.adobe.com/illustrator/using/workspace-basics.html), panel groups) | done | `window.panel.collapse {panel, collapsed?}`; in the legacy dock and in the shared dock's tree (floating groups already did); a group beside another (a left/right split) doesn't collapse |
| Panels outside the app window (another monitor) | missing | single OS window |
| Edit Toolbar (•••): the All Tools drawer opens beside the toolbar by its button (Escape or a click outside closes it), dims the tools already in the toolbar; while it is open, tools drag from it onto the toolbar (between two tools, or onto one to join its group), from the toolbar into it, and within the toolbar; each toolbar keeps its edits | done | #1018; `window.editToolbar`, `toolbar.addTool`/`removeTool`/`reset`; the drawer's Reset (Illustrator has it in a drawer menu); dragging a toolbar button moves the tool it shows, not its whole group; New/Manage Toolbars missing; tool flyouts are as wide as their longest translated name and shortcut need (#1616) |
| Quick Image Trace presets trace at once and leave the panel as it was; the Image Trace button (Control bar, Properties) opens the editable controls | done | Per the public documentation, choosing a preset traces the image with it straight away, and the panel is opened with the Image Trace button in the Control panel or in Properties' Image Trace section (<https://helpx.adobe.com/illustrator/using/image-trace-results-optimization.html>). The controls show Threshold, Paths, Corners and Noise with Advanced open; `panels/image_trace.rs` headless tests (`a_quick_trace_leaves_the_panel_as_it_was`, `sketch_quick_trace_reveals_all_adjustments_and_keeps_the_source` for the button's route). No algorithm parity claim. |
| Image Trace panel: holding the eye button next to View down shows the selected Image Trace objects' source image on the canvas | done | Per the public documentation, the control next to View is pressed and held to compare the trace result with the source image (<https://web.archive.org/web/20250831090623/https://helpx.adobe.com/illustrator/using/image-trace.html>). On release the objects show their View again; the document, its View and the undo history are unchanged. `RenderOptions::trace_source`; `panels/image_trace.rs` headless test `holding_the_eye_button_shows_the_source_image_until_released`, render test `trace_source_draws_the_source_image_of_the_listed_objects`. The Control bar and Properties have no such button |
| Live trace result bounds and selection highlights exclude the retained bitmap; source-image views retain it | done | All TraceView modes and rotated result bounds regression-covered; ordinary hidden group behavior retained |
| Image Trace panel: the Preview checkbox before Trace chooses whether a setting change traces the selected Image Trace object again at once | done | Per the public documentation, Preview shows the result of the current settings and Trace traces with them (<https://web.archive.org/web/20250831090623/https://helpx.adobe.com/illustrator/using/image-trace.html>). Preview is on by default and kept with the UI preferences; Reset to Default leaves it. Off, changes wait in the panel until Trace applies them as one undo step, or until Preview is turned on again. `ui.set {imageTracePreview}`; `panels/image_trace.rs` headless tests (`with_preview_off_a_slider_change_waits_for_trace`, `turning_preview_on_traces_the_changes_that_waited`, `preview_turned_on_with_nothing_selected_traces_when_the_object_is_selected_again`, `settings_that_wait_stay_with_their_object`, `preview_is_kept_with_the_preferences_and_reset_to_default_leaves_it`) |
| Toolbar without group labels by default, one or two columns; All Tools drawer by flyout group with shortcuts, its menu (Reset, Basic, Advanced) and Show section | done | #1751; a preference brings the labels back; `toolbar.show` sets the Fill/Stroke, Coloring, Draw Mode and Screen Mode controls |
| Actions panel: a clicked action stays selected for Play Current Selection and Delete Selection | partial | #1853 (part of #1629); editing actions and action shortcuts remain |
| Actions panel: Create New Set (bottom bar, ≡ › New Set…) names a new set and Set Options… renames the current one; a click on a set's name makes it current and a recording goes into the current set ([helpx: Automation with actions](https://helpx.adobe.com/illustrator/using/automation-actions.html)) | partial | #3712; the New Action dialog (name, set menu, shortcut, colour), dragging actions between sets and deleting a set remain |
| Properties panel per context (every selection type's Quick Actions) | partial | one clip group: Edit Clipping Path and Edit Contents, at the top of the panel rather than in Quick Actions (G1); the overview's additional controls ([Properties panel overview](https://helpx.adobe.com/illustrator/desktop/get-started/learn-the-basics/properties-panel-overview.html)): Blend Options (one blend, Blend tool) and Merge Live Paint (a Live Paint group with more) in Quick Actions, the Width Profile list (Width tool or a variable-width stroke) and an applied effect's Delete icon in Appearance, Corner and Corner Radius for corners selected with Direct Selection; Variable Font was already there |
| Select › Start Global Edit / Stop Global Edit; Properties › Quick Actions with the Global Edit Options popover (Match: Appearance, Size; On Artboards: All Artboards or a range, Include Objects On Canvas; Result(s)); Escape stops it; the matched objects are highlighted in their layer colour while it runs | partial | #3521, `select.globalEdit` / `select.globalEditOptions`, `crates/engine/src/tests_globaledit.rs`; paths, compound paths and symbol instances; the Control panel's copy of the options, Shift-click to leave a match out, highlighting matches before the start and similar groups remain ([public source](https://helpx.adobe.com/illustrator/desktop/manage-objects/edit-objects/edit-similar-objects.html)) |
| Contextual task bar | done | |
| Control bar with Transform and Stroke popovers | done | |
| Pathfinder shape results keep their source names and SVG IDs | done | #1293; Minus Front keeps the back name, Minus Back the front name; Unite/Intersect/Exclude use the backmost named input; compound output carries it once; geometry and paint unchanged |
| Attributes fill-rule controls with a compound member selected | done | Edits the compound's rule; direction still edits only the member; rendered-hole and Undo/Redo regression tests (#1197) |
| Make Compound Path on a group (such as outlined type) keeps its backmost path's paint; compound paths joined into one keep their holes | done | #1537 |
| Symbol libraries: Window › Symbol Libraries (Basic Shapes, User Defined, Other Library…, Save Symbols…) and the Symbols panel's Symbol Libraries Menu button and its menu's Open Symbol Library / Save Symbol Library…; a click on a library symbol adds it to the document's Symbols (one undo step, it becomes the current symbol), Shift/Cmd-clicks select for Add to Symbols, and a symbol dragged onto the canvas is added and placed where it is dropped | done | [public source](https://helpx.adobe.com/illustrator/desktop/manage-objects/traces-mockups-symbols/create-or-import-symbol-libraries.html), [Symbols panel options](https://helpx.adobe.com/illustrator/desktop/manage-objects/traces-mockups-symbols/symbols-panel-options.html), [placing from a library](https://helpx.adobe.com/illustrator/desktop/manage-objects/traces-mockups-symbols/create-and-place-symbols.html); `window.symbolLibrary`, `window.symbolLibrary.other`, `ui.saveSymbolLibrary` (dialog `saveSymbolLibrary`), `window.userSymbolLibrary1`…`10`; `panels::tests_symbol_libraries`; only one built-in library (original, generated in code) |
| Symbols panel: New Symbol opens Symbol Options (Name) before making the symbol; Alt-click skips it | partial | #1560 (#1566); Registration 9-point grid in Symbol Options (`symbol.new` / `symbol.place`); Export Type (Graphic / Movie Clip) and Enable Guides for 9-Slice Scaling (guides stored, not drawn on the canvas); Dynamic/Static symbol type still missing |
| Pathfinder Trim, Merge and Crop remove strokes; Merge joins same-fill objects whatever their strokes | done | #1570 (#1572) |
| Pathfinder panel on one group selected alone; a greyed-out button says what it needs | partial | #1450; the group's members are combined; one plain path selected alone stays disabled (the public documentation doesn't say what the operations do to a single path) |
| Pathfinder Divide: an open cutting line splits a filled path | done | #1307; straight, crossed, bent and curved cutters, holes and frontmost appearance regression-tested; native automation: independent piece movement, one Undo/Redo, saved-document round trip and window capture; existing bounded-map fallback remains |
| Canvas context menu | done | Layers rows, panels and type have none (gap) |
| SVG Interactivity panel: JavaScript for each object's events and the document's linked JavaScript files, written to SVG and read back as text | done | `svgInteractivity.*`, `ui.javascriptFilesDialog` |
| Retype, Comments, Version History panels | missing | the last two are cloud features (out of scope) |
| Blend panel (30.7) | partial | Window › Blend provides Type (Ease In, Ease Out, Ease In Out, Ease Out In), Ramp 0–100% and independent color Shift, alongside the existing blend controls. Easing is disabled without a spine. Original polynomial distributions approximate the publicly documented behavior; on-canvas easing handles remain missing. [Public source](https://helpx.adobe.com/illustrator/desktop/manage-colors/apply-transparency-and-blending/blend-panel-overview.html) |
| Gradient panel: Dither (30.0) | done | Dither box under the stops (`paint.editGradient` `dither`); dimmed behind the Stroke proxy, fills only; `panels::gradient` test |
| Gradient panel: Method, Classic or Perceptual (30.0) | done | Method menu under the Stroke buttons for linear and radial gradients (`paint.editGradient` `method`); [public source](https://helpx.adobe.com/illustrator/desktop/paint-and-fill/create-and-edit-gradients/control-perceptual-interpolation-in-gradients.html); `panels::gradient` test |
| SVG Interactivity, Retype, Comments, Version History panels | missing | the last two are cloud features (out of scope) |
| Workspaces (save, reset, switch); each remembers the layout changes made in it across switches and restarts until Reset <name> | done | `UiState::workspace_changes` |
| Background Export: a progress dialog with the file name, the artboard count and a bar (busy for one artboard and for formats made in one go), Cancel (Esc) and Continue in Background; it opens after 0.4 s and never over another dialog. A cancelled export writes no file | done | #2564; on the shared `vectorcraft_doc::cancel` token (#2546), so a cancelled render keeps no half-drawn pattern tile or effect raster. The bar moves between artboards, not within one. Save As, Export for Screens and Export Selection don't show it yet |
| Swatch, graphic style and symbol library panels: each library opens in a panel of its own, several at once, that floats, groups, docks and closes like any panel; opening an open library brings its panel forward; Previous / Next Library change the library in place; the panel menu's Persistent reopens the panel at startup with its view and find field (a library loaded from a file is read again; when its file is missing the panel reopens showing the file's path with Locate Library…, and the status bar names the file) | done | #2603; panel ids `library:<kind>:<id>`, `ui.libraryPanel {panel, library?, view?, showFind?, persistent?}`, `ui.libraryPanel.locate {panel, path?}`; the library panels have no sort option, as before |
| DXF export/import preserves intended curve closure | done | #1299; R13–2018 closed SPLINE bit set/read; an open curve ending at its start stays open; knots and control points unchanged; native CIRCLE/ARC export and CNC preset remain pending |
| macOS save sheet: choose the format from a menu | done | #866; the save panel is VectorCraft's own `NSSavePanel` with its Format menu set before the sheet opens (`apps/vectorcraft/src/mac_save_format.rs`); #1855 had withdrawn the menu that was added after the sheet began, which made Save As, Save a Copy and the first Save show no sheet |
| Preferences: every option acts | partial | ~16 stored but unused (#394) |
| Preferences › General › Constrain Angle | done | arrow keys and Shift drags along the turned axes; the Rectangle, Rounded Rectangle, Ellipse and grid tools, point and area type and the shape size dialogs draw along them with the bounding box turned; Shift-drawn polygons and stars keep the turned base orientation; the Line Segment tool's Shift steps count from it (#394, <https://helpx.adobe.com/illustrator/desktop/manage-objects/arrange-objects/rotate-objects-by-specific-angles.html>); arcs, spirals, flares and Pen paths are not turned |
| Tabs panel: a stop drags from where the press landed, a stop just added included | done | #1458 |
| Outline view keeps template layers in Preview, dimmed | done | #1486; a picture placed to be traced stays visible under the outlines |

## Look and feel

- [x] Recolor Artwork › Assign: double-click a row's new color to pick it in the Color Picker, in the model it was picked in (part of #4028); a Lab new color is kept but can't be typed yet (the picker's Lab fields work in RGB); Lab entry, swatch libraries and the row menu remain.

- [x] Recolor Artwork Edit color chips scroll within the screen, preserving the wheel and actions (#3416); 5, 20, 200 and 400 colors, resizing and UI scaling, final-chip editing and preview/history covered by egui_kittest.

- [x] Status messages and bounded readouts remain stable as values, canvas hover and background jobs change; ordinary values fit all registered languages and units, and optional groups yield in narrow windows (regressions; contributor rebuilt and confirmed the original-file status-bar scenario on 2026-10-19). Sample widths are cached per UI and refreshed only when language, document units or pixels per point changes.

- [x] Cached canvas dimensions match the renderer limit before cache-key creation, view construction and worker submission (controlled-worker regression).
- [x] Cached canvas frames keep their document scale during panel/window resize, including rotated views and DPI changes (controlled-worker regression; contributor rebuilt and confirmed the original-file resize scenario on 2026-10-19).
Medium Dark theme and four brightness levels, the categorized and Advanced toolbars, 35 pt document tabs,
33 pt panel tabs, hint bar, contextual task bar and native macOS menus, restyled to values measured from
public screenshots on 2026-10-02 (~75–80% then). Missing: a System theme that follows the OS (#825), a
re-measure of the panels added since. Slider rails show on Medium Dark (#1808, #1662).

- [x] Native saves restore Show Transparency Grid (#1968), without dirtying the document or adding an undo step.

- [x] Compact Color Picker with a distinct header, stacked actions and aligned input groups. #2008; translated action widths, visible warnings, stable mode switching and vertical scrolling in narrow windows; real egui events and two UI scales covered; German and enlarged English renders reviewed locally. Scores unchanged

## Highlighted-text Eyedropper (2026-10-21)

Type/Eyedropper switches preserve highlighted characters; sampling changes only their enabled
font/paint attributes. Enabled non-neutral source or target object transparency is refused
atomically; disable Transparency to preserve object transparency while sampling the range.
`tests_eyedropper.rs` covers ranges, Unicode, history and transparency; tool tests cover brush-key
sampling coexistence. Scores and estimates unchanged. See [development.md](development.md#highlighted-text-eyedropper-2026-10-21).

## Drag artwork between document tabs (2026-10-22)

Selection drags activate another visible document tab after 400 ms, then copy editable artwork
on canvas release with the original grab point. Source and clipboard preservation, resource
rendering, swatch conflicts, cancellation, rotated/zoomed views and target Undo/Redo have regression
coverage (`tests_document_drag`, `tests_clipboard`). Overflow-menu and cross-window drag remain
separate. Scores and hours unchanged. [Behavior and command details](development.md#drag-artwork-between-document-tabs-2026-10-22).

## Revision history

| Date | Change | Summary |
|---|---|---|
| 2026-10-24 | minor | Recolor Artwork › Assign: double-clicking a row's new color opens the Color Picker over the dialog; its OK sets the row's new color in the model it was picked in (a CMYK mix stays CMYK, RGB becomes CMYK in a CMYK document), Cancel leaves the row as it was. A Lab new color is kept, but a Lab color can't be typed yet: the picker's Lab fields work in RGB, as for fill and stroke (part of #4028; Lab entry, swatch libraries and the row menu remain). Scores and estimates unchanged. |
| 2026-10-24 | minor | Properties: Merge Live Paint shows only when the selection holds nothing Live Paint refuses (type, images and other objects that aren't paths), which `livePaint.merge` and Make now refuse too. Scores unchanged. |
| 2026-10-24 | minor | G1: Properties has the additional contextual controls of the public Properties panel overview: Blend Options and Merge Live Paint in Quick Actions, the Width Profile list and an applied effect's Delete icon in Appearance, and the kind and radius of corners selected with Direct Selection. Scores unchanged. |
| 2026-10-24 | minor | Docs catch-up: Relative rounding (#3091) marked done; the stale "Video rulers: missing" duplicate row removed (done, #2849). Scores and estimates unchanged. |
| 2026-10-23 | minor | Actions panel: Create New Set and Set Options… name action sets, and a recording goes into the set clicked or just made (#3712). Scores and estimates unchanged. |
| 2026-10-23 | minor | Select › Start Global Edit / Stop Global Edit (menu label flips), Properties › Quick Actions with the Global Edit Options popover (Match: Appearance, Size; On Artboards: All Artboards or a range, Include Objects On Canvas; Result(s)), Escape stops it, the matches are highlighted on the canvas (#3521). Scores and estimates unchanged. |
| 2026-10-23 | minor | Symbol libraries: Window › Symbol Libraries and the Symbols panel's Symbol Libraries Menu open library panels (Basic Shapes, User Defined, Other Library…); click adds, drag places, Save Symbols / Save Symbol Library writes a library (`panels::tests_symbol_libraries`). |
| 2026-10-23 | minor | Selection tool: every selected area type and type on a path shows its in and out ports in their states (empty, arrow, red "+"); the out port of type that fits loads the text too, and the in port threads a new or clicked frame before the clicked one (#2572). Scores and estimates unchanged. |
| 2026-10-23 | minor | Smart guides: Distance Guides (Alt/Option over another object or an empty part of an artboard with the Selection tool) and the Smart Guides › Invoke Distance Guides preference; scores and estimates unchanged. |
| 2026-10-23 | minor | Double-clicking the Line Segment, Arc, Rectangular Grid or Polar Grid tool opens its Tool Options (#2691); the grid dialog row is done; scores and estimates unchanged. |
| 2026-10-23 | minor | Pencil: Shift held at the press draws a straight line at 45° steps from the Constrain Angle (part of #3799). Scores and estimates unchanged. |
| 2026-10-23 | minor | Fix #3705: in Draw Inside, art drawn or pasted inside the isolated object takes isolation mode into its clip group, so it can be clicked and isn't dimmed (it used to stay locked and dimmed beside the isolated path); undo or redo that removes the isolated object ends isolation mode; scores and estimates unchanged. |
| 2026-10-23 | minor | Preferences › General › Constrain Angle: arrow keys, Shift drags and new rectangles, ellipses, grids, type, polygons, stars and size-dialog shapes follow it (#394). Scores and estimates unchanged. |
| 2026-10-22 | minor | Pen, Anchor Point and Direct Selection drags: the preview gets every pointer position, including where it re-enters the window and the move in the release's frame (#834); score unchanged (estimated) |
| 2026-10-23 | minor | Guides: a path moved by a press on its stroke or beside its edge, and a straight segment Direct Selection drags, snap by the nearest point of the path, landing exactly on a ruler guide (#3701); scores and estimates unchanged. |
| 2026-10-23 | minor | Fix #3705 in part: drawing modes apply to pastes (Draw Inside puts Paste, Paste in Place and Paste on All Artboards inside the object drawn into; Draw Behind puts Paste in Place and Paste on All Artboards behind the selection), and Shift+D goes from Draw Behind back to Draw Normal when Draw Inside can't draw into the selection instead of stopping at Draw Behind; scores and estimates unchanged. |
| 2026-10-23 | minor | Artboard Options: Show Video Safe Areas (action-safe 3.5% and title-safe 5% in from each edge, EBU R 95, canvas only) and Video Ruler Pixel Aspect Ratio, per artboard; View › Rulers › Show Video Rulers counts the active artboard in those pixels (#2849); two rows done; scores and estimates unchanged. |
| 2026-10-23 | minor | Artboard Options: Show Center Mark and Show Cross Hairs (`artboard.setProps`/`artboard.new` `showCenterMark`, `showCrossHairs`) draw a center cross and lines from the middle of each side through the center on the canvas only, in one undo step; video safe areas, video rulers and the pixel aspect ratio remain missing; scores and estimates unchanged. |
| 2026-10-23 | minor | Simplify: Object › Path › Simplify opens its slider at the Auto-Simplify level for the selection, previewing that result, instead of at 1 pt; partial-path simplify remains missing; scores and estimates unchanged. |
| 2026-10-23 | minor | Simplify of part of a path: with anchors or segments of a path direct-selected, Object › Path › Simplify (slider, Auto-Simplify and dialog) refits only each run of selected segments between its end anchors; the Simplify row is done (its stale duplicate removed); scores and estimates unchanged. |
| 2026-10-22 | minor | Gradient panel: the Method menu, Classic or Perceptual (Illustrator 30.0); scores and estimates unchanged. |
| 2026-10-22 | minor | Gradient panel: the Dither box (Illustrator 30.0); scores and estimates unchanged. |
| 2026-10-22 | minor | Recolor Artwork Edit chips scroll within the available screen height (#3416); small palettes retain their tab height; scores and estimates unchanged. |
| 2026-10-22 | minor | Artboard Options honors Scale Artwork with Artboard for preset, orientation and typed-size changes; both Fits keep artwork stationary; scores and estimates unchanged. |
| 2026-10-22 | minor | Gradient panel: the Method menu, Classic or Perceptual (Illustrator 30.0); scores and estimates unchanged. |
| 2026-10-22 | minor | Gradient panel: the Dither box (Illustrator 30.0); scores and estimates unchanged. |
| 2026-10-21 | minor | Blend panel Type and Ramp control step easing with a separate color Shift (#2539); Blend Options retains spacing and orientation; polynomial curves are original approximations. Scores and estimates unchanged. |
| 2026-10-21 | minor | Workspaces keep the layout changes made in them when switching away and back, and across restarts; Reset <name> returns to the saved layout; score unchanged (estimated) |
| 2026-10-21 | minor | Panels dragged by the mouse join a dock group as tabs, split it top/bottom or become icons in the icon column; dock tabs close from their context menu; score unchanged (estimated) |
| 2026-10-21 | minor | A double-click on a dock tab collapses its group to its tabs, as floating groups already did, and expands it again; another of its tabs expands it (`window.panel.collapse`); scores unchanged (estimated) |
| 2026-10-20 | minor | Workspaces keep the layout changes made in them when switching away and back, and across restarts; Reset <name> returns to the saved layout; score unchanged (estimated) |
| 2026-10-21 | minor | M3: highlighted-text Eyedropper sampling preserves ranges and rejects unsupported source/target transparency atomically; regression-tested; scores and estimates unchanged. |
| 2026-10-21 | minor | Shape Builder Gap Detection shares Live Paint’s spatial gap finder and temporary merge bridges (#2526); Custom is capped at 72 pt in the existing options dialog. Scores and estimates unchanged. |
| 2026-10-21 | minor | Tangent and straight-segment perpendicular snapping use saved Smart Guides Geometric Guides preferences and Control bar quick access (#2537); no View submenu. Scores and estimates unchanged. |
| 2026-10-21 | minor | 3D: original Standard, Diffuse, Top Left and Right lighting presets; command access, effective-setting recognition and preview/history; scores and estimates unchanged |
| 2026-10-21 | minor | G1: one selected clip group's Properties panel has Edit Clipping Path and Edit Contents at its top (public doc cited in target-app-parity.md). Scores unchanged. |
| 2026-10-19 | minor | Distribute Objects and Distribute Spacing space the selected ruler guides across the axis with the objects, as items with no size, and two or more guides selected alone (#1597 in part); with a key guide and a spacing, the guide stays put and the other items are spaced from it (key guide from #3165); the Distribute Spacing value is enabled for a key guide; scores unchanged |
| 2026-10-21 | minor | Snap to Glyph marked done (#3124); scores unchanged |
| 2026-10-20 | minor | The selection tools (Selection, Direct Selection, Group Selection) take a clipping path on its outline, within the selection tolerance, before the art it clips, so its segments can be dragged where the art covers the mask's edge; Select Behind lists the same, and the eyedropper and the tools that act on what is drawn are unchanged. Doc and tool regression tests. Scores unchanged. |
| 2026-10-20 | minor | Polygon and Star exact-size placement remembers the chosen count for later drags and dialog openings; Cancel preserves the previous count; existing opening gestures retained; scores and hours unchanged. |
| 2026-10-20 | minor | Live trace result bounds and selection highlights exclude the retained bitmap except in source-image views; scores and hours unchanged. |
| 2026-10-19 | minor | A ruler guide selected with objects can be the key object (#2143): the Selection tool's click on it, or Align to Key Object with one object; drawn thicker; Align lines the objects up on it and the guide stays put; the other way round, selected guides align to a key object, which one object selected with guides can be; scores unchanged |
| 2026-10-19 | minor | Cache status-readout sample widths per UI, language, document unit and pixels per point; regressions check reuse across live frames and refresh after display-setting changes. Scores and estimates unchanged. |
| 2026-10-19 | minor | Status messages and readouts use stable responsive allocations sized for localized labels and ordinary coordinates. Contributor original-file acceptance confirmed; scores and estimates unchanged. |
| 2026-10-19 | minor | Cached canvas dimensions match the renderer limit before cache-key creation, view construction and worker submission. Regression coverage; scores and estimates unchanged. |
| 2026-10-19 | minor | Cached canvas reprojection preserves scale during resize, rotation and DPI changes. Contributor original-file acceptance confirmed; scores and estimates unchanged. |
| 2026-10-19 | minor | Color Picker layout (#2008): compact controls, centered header, stacked actions and aligned inputs; screen-reader channel values, current shared button names and styling, screen sampler and model-by-model Tab order retained. Narrow windows scroll vertically; scores and hours unchanged. |
| 2026-10-18 | minor | Mirror Repeat shows its axis when selected (#2877): drag the line or center handle to move it, drag an end handle to rotate it (Shift: 45°), Escape cancels and each drag is one undo step. Engine and headless canvas gesture regressions passed; native move, rotation and Escape screenshots reviewed. Formatting, Clippy, WASM and repository checks passed; the undo label reuses translated Repeat Options. (#2877) Scores and hours unchanged. |
| 2026-10-18 | minor | Bounding-box handles take priority where a selected Mirror Repeat axis crosses them: the Top handle scales the repeat, while the center still moves it and the ends still rotate it. A headless canvas regression confirms height changes while width stays fixed; scores and hours unchanged. |
| 2026-10-19 | minor | Blend Options: Ease In / Ease Out and Color Ease In / Out sliders with live preview (#1124); the Blend panel row stays partial (no easing in the panel, no on-canvas handles) |
| 2026-10-19 | minor | Smart guides and snapping: tangent and perpendicular snapping for lines drawn from an anchor and View › Snapping (#2530), both partial; scores and hours unchanged |
| 2026-10-18 | minor | Status-bar pointer X/Y follows the active global/artboard ruler origin and General units; three painted-text regressions cover origins, mode/artboard/reset/unit changes and read-only state; scores and hours unchanged. |
| 2026-10-17 | minor | Blend spine Anchor Point conversions, selection/stacking priority and Shift regression coverage (#2544); scores and hours unchanged. Windows native canvas checks and screenshots reviewed in PR #2643. |
| 2026-10-17 | minor | Artboard Options has X and Y beside a reference point locator, counted from the ruler origin as the Transform panel's are; a typed X or Y moves the artboard's reference point there and a new Width or Height keeps it in place (#1876); scores unchanged (estimated) |
| 2026-10-17 | minor | Background Export shows a progress dialog with Cancel and Continue in Background (#2564), built on the shared cancel token of #2546; scores and hours unchanged (estimated) |
| 2026-10-17 | minor | Library panels (#2603): each swatch or graphic style library opens in its own dockable, tabbable panel, several at once; the panel menu's Persistent reopens a panel at startup with its view and find field, a library loaded from a file read again from its path; scores unchanged (estimated) |
| 2026-10-17 | minor | Selection tool: clicking the red out port of overflowing area type, then a closed path, empty canvas or a dragged rectangle threads the text there (#2572 in part); Escape cancels. Scores and hours unchanged. |
| 2026-10-17 | minor | Group Selection picks one path of a compound path (the outer shape or a hole) and moves it on its own; a second click adds the compound path (#2441, in part) |
| 2026-10-16 | minor | The Move dialog opens on the offset of the last drag or Move and with the checkboxes it was last confirmed with; scores unchanged |
| 2026-10-16 | minor | Native documents restore transparency grid visibility (#1968); save/open regression covers on/off, old files and SVG; scores and hours unchanged (estimated) |
| 2026-10-16 | minor | Live Corners: a corner rounds until its cut reaches the neighbouring anchors or their cuts, so one corner of a 100 pt square alone reaches a 100 pt radius instead of 50 (#2332); score unchanged (estimated) |
| 2026-10-16 | minor | Correction: quick Image Trace presets trace at once and leave the panel as it was (the 2026-10-14 row below said a quick preset reveals it; the later change in `baadbe2c6` made the button the way in); the Image Trace button in the Control bar or Properties opens the controls, as the public documentation describes. Wording only; scores and hours unchanged. |
| 2026-10-16 | minor | Batch exporting in Export for Screens (#1935): a file name pattern (document and item names, a counter, the date), folders per format, item or both, Layers and Selection tabs with thumbnails, EPS, AI and VectorCraft rows, a PDF row with printer's marks, and export presets (built-in Logo Package and Print-Ready PDF, saved ones); scores and hours unchanged (estimated) |
| 2026-10-16 | minor | Anchor Point tool: a click on one handle end retracts that handle only; the anchor becomes a corner and keeps the other handle (#2267). Handle drags start past the same 2 px as anchor drags. Scores and hours unchanged. |
| 2026-10-16 | minor | Smart Guides: the dragged end of a straight segment snaps to the segment's extension at any angle, with the guide from its other end (#2120); scores and hours unchanged |
| 2026-10-15 | minor | Cursors of their own for 36 more tool states: node-editing cursors as the public manual describes them, Anchor Point and Curvature, Free Transform's side-shear; the rest borrow a drawn cursor until they're drawn. Busy cursor listed as missing. Every cursor is listed in `crates/tools/src/cursors.toml` |
| 2026-10-16 | minor | 3D lighting: one to ten directional lights, a selectable light list, Add Light and Delete Light, independent settings and artboard gestures; ambient applied once and smooth RGB vector export; scores and hour estimates unchanged |
| 2026-10-16 | minor | 3D lighting: opaque RGB Color for the shared directional light, with neutral ambient, preview/reset controls and smooth vector Revolve shading; scores and hour estimates unchanged |
| 2026-10-14 | minor | Effect > Warp: style, orientation and percent controls; focused checks passed and original captures reviewed; scores and hours unchanged |
| 2026-10-14 | minor | G11: open-path anchor Delete leaves separate surviving runs; dedicated Remove Anchors remains connected. Keyboard and history regression coverage added; scores and hours unchanged. |
| 2026-10-14 | minor | Curvature: the Curvature tool's curve: each stretch of three points follows the circle through them (handles along its tangent, chord/(3·cos²(α/2)) long, shorter in a turn tighter than
Sign in
Home
Shorts
Subscriptions
You
History
Sign in to like videos, comment, and subscribe.
Sign in
Explore
Music
Movies & TV
Live
Show more
More from YouTube

  
    
    
  
YouTube Premium

  
    
    
    
  
YouTube Music

  
    
      
      
    
    
      
      
    
  
YouTube Kids
Report history
AboutPressCopyrightContact usCreatorsAdvertiseDevelopersImpressumCancel Memberships
TermsPrivacyPolicy & SafetyHow YouTube worksTest new features
© 2026 Google LLC
All
Shorts
Unwatched
Watched
Videos
Recently uploaded
Live
About these results
Filters
Shorts
Step 1 for Monstera deliciosa: The BIG chop to propagate #monsteraplant #plantpropagation
1.6M views
Monstera deliciosa - Thai Constellation
30K views
This is how I grew my monstera deliciosa from seed! #timelapse #monstera
767K views
Never underestimate a well grown classic #monsteradeliciosa grown by @plantsdecoranddiy
299K views
The MOST DANGEROUS Fruit? - The Monstera Deliciosa Fruit!
229K views
true
11:21
Now playing
If I Only Knew These Monstera Tips 5 Years Ago
2M 3y ago
Sheffield Made Plants
My best Monstera care tips I wish I knew 5 years ago. -------------------- Download my FREE Plant Parent's Troubleshooting ...
9:12 ... actually two types of monstera plants you can buy that look very similar the monstera bozigiana and the monstera deliciosa they ...
4K
true
4:28
Now playing
Do this after you buy your first Monstera
204K 2y ago
Kill This Plant
Download My Houseplant App - https://blackthumbplant.app/ Join Our Discord for plant help - https://discord.gg/kw9FnABCRk ...
4K
Summary
Kill This Plant analyzes why a new Monstera may struggle by measuring light levels and soil moisture. Discover how to identify if a plant is receiving enough light and why adjusting the environment immediately after purchase is crucial for healthy growth.
true
8:12
Now playing
How to grow the BEST Monstera
1.3M 2y ago
Kill This Plant
Download My Houseplant App - https://blackthumbplant.app/ Join Our Discord for plant help - https://discord.gg/kw9FnABCRk ...
1 product
4K
Summary
Kill This Plant compares top cuttings versus rearranging existing vines to fix crawling Monsteras. Learn methods for staking, pruning for propagation, and managing soil requirements for healthier, upright growth.
true
4:06
Now playing
how to grow a monstera as fast as possible
57K 9mo ago
Kill This Plant
Download My Houseplant App - https://blackthumbplant.app/ Join Our Discord for plant help - https://discord.gg/kw9FnABCRk ...
4K
Summary
Kill This Plant explains how to optimize monstera growth by mastering water saturation, soil drainage mixtures, and light intensity. Learn to use tools for testing soil moisture and tips for extending daylight hours to ensure consistent, rapid plant development.
true
8:01
Now playing
7 MONSTERA Plant Care Tips You Need to KNOW : Monstera Deliciousa
340K 3y ago
MonstroFarm
I want to help you grow beautiful Monstera's, which is Super-Easy. Here are 7 Plant Care Tips on how I care and make sure that ...
8 chapters
Intro | LIGHTING | TEMPERATURE AND HUMIDITY | SOIL MIXTURE | FERTILIZER | PRUNING | PEST MANAGEMENT | WATERING
true
2:42
Now playing
239 Days Monstera Deliciosa Time-Lapse – From Seed to Stunning Plant! 🌱
83K 1y ago
Interesting as FCK
Ever wondered how long it really takes to grow a Monstera deliciosa from seed? Here's the full 239-day journey captured in one ...
4K
true
10:22
Now playing
0:19
10:22
Set Up Your Monstera For Success
263K 2y ago
Sheffield Made Plants
Download my FREE Plant Parent's Troubleshooting Handbook https://resources.sheffieldmadeplants.com/handbook ▸ 1:1 ...
4K
Matching chapter
0:19 My Monstera
true
3:13
Now playing
why people keep Killing their Monstera
1M 3y ago
Kill This Plant
Hopefully this video will get you to stop killing your Monstera Deliciosa. *Links in this description contain affiliate links.
0:00 Monstera deliciosa is supposed to be one of the easiest plants but you would not believe how many people are asking for help but ...
4K
true
5:18
Now playing
Why you need to cut your monstera
337K 2y ago
Kill This Plant
Download My Houseplant App - https://blackthumbplant.app/ Join Our Discord for plant help - https://discord.gg/kw9FnABCRk ...
4K
Summary
Kill This Plant demonstrates the process of air layering a large Monstera to safely propagate new roots before cutting. This technique allows for maintaining large foliage while preparing the plant for a fresh start in a new pot.
true
49:16
Now playing
I Wish I Knew These Monstera Tips 5 Years Ago
562K 2y ago
Sheffield Made Plants
All my best Monstera tips in 1 video! -------------------- Download my FREE Plant Parent's Troubleshooting Handbook ...
0:00 Perforations or holes in the Le or monstera deliciosa is one of the most highly Sal offer Commodities in the world sort of if you ...
4K
true
7:22
Now playing
Do These 3 Things For The Largest Monstera Deliciosa
70K 3y ago
Techplant
How to grow monstera deliciosa the right way for large leaves. These tips will help you grow a really nice monstera deliciosa!
4K
Summary
Techplant clarifies common misconceptions about pruning lower leaves and recommends single-stalk configurations for easier management. Discover the importance of proper orientation when securing plants to supports, ensuring the aerial roots are pressed against the pole for maximum leaf growth.
true
12:30
Now playing
The World’s Largest Monstera Deliciosa Plant
62K 3y ago
SleepyLizard
Get your Monstera Delciosa fruit and avocados at: https://www.guacfarm.com.
Summary
SleepyLizard showcases a massive Monstera Deliciosa plantation focused on fruit production rather than decoration. The video explores harvesting the sweet fruit and the agricultural challenges of integrating these plants within an avocado grove.
true
11:06
Now playing
10 Must Do Tricks For The BEST Monstera
714K 3y ago
Sheffield Made Plants
Head to https://squarespace.com/sheffield to save 10% off your first purchase of a website or domain using code SHEFFIELD.
4K
true
10:59
Now playing
How To Propagate Your Monstera The Right Way
347K 2y ago
Sheffield Made Plants
Monstera propagation the right way -------------------- Download my FREE Plant Parent's Troubleshooting Handbook ...
0:00 This is The Ultimate Guide to propagating and multiplying your monstera deliciosa or swiss cheese plant so you can have more of ...
4K
true
16:00
Now playing
Why You shouldn’t buy a Monstera Deliciosa !
15K 1y ago
The Millennial Plant Dad
The Monstera Deliciosa may be an iconic house plant staple but the fact of the matter is that it's not for everyone, home or space !
4K
7 chapters
Intro | Space | How it is sold | M poles | Climbing | Fertilizing | Outro
true
7:34
Now playing
Eating The Tastiest Fruit In The World! - How to Eat Monstera Deliciosa Safely | Mexican Breadfruit
657K 3y ago
Greenthumbs Garden
Monstera Delcicosa is a popular houseplant all over the world. What many do not know is that when grown outside in the right ...
CC
6 chapters
The Monstera Delcicosa Fruit | Monstera Delcicosa Fruit Nutritional Benefits | When To Harvest Monstera Delcicosa Fruit | When Is It Safe To Eat Monstera Delcicosa Fruit | What To Do With Unripe Monstera Delcicosa Fruit | Tasting The Monstera Delcicosa Fruit
true
11:05
Now playing
Grow Your Pothos 3x FASTER
768K 2y ago
Sheffield Made Plants
Download my FREE Plant Parent's Troubleshooting Handbook https://resources.sheffieldmadeplants.com/handbook ▸ 1:1 ...
4K
Summary
Sheffield Made Plants shares techniques for maximizing pothos growth through strategic potting, stem pinning, and propagation methods. Learn how proper lighting, soil selection, and understanding root health contribute to fuller, faster-growing vines.
true
12:48
Now playing
Popular Plant Products I’ll NEVER Buy Again (And What I Use Instead)
272K 1y ago
Sheffield Made Plants
Download my FREE Plant Parent's Troubleshooting Handbook https://resources.sheffieldmadeplants.com/handbook ...
4K
10 chapters
Intro | Rooting Hormone | Miracle Grow | Clear Pots | Decorative pots | Grow lights | Velcro plant eyes | Leaf shine | Core poles | Scissors
true
44:26
Now playing
Inside an Indoor Plant Oasis | Full Tour
325K 1y ago
Sheffield Made Plants
Download my FREE Plant Parent's Troubleshooting Handbook https://resources.sheffieldmadeplants.com/handbook ...
4K
Summary
Richard provides a comprehensive look into a living space transformed into a dense indoor jungle featuring over 200 diverse house plants. The tour showcases popular species alongside unique finds, sharing propagation tips and creative styling methods for various leaf types.
true
10:21
Now playing
Changing THIS Will Make Your Spider Plant THRIVE
914K 1y ago
Sheffield Made Plants
Download my FREE Plant Parent's Troubleshooting Handbook https://resources.sheffieldmadeplants.com/handbook ...
4K
6 chapters
Intro | Leaf Problems | Why Does This Work | Fertilizer | pH | Water Conditioner
true
12:26
Now playing
Growing A MATURE MONSTERA | How Long Does It Take???
26K 3y ago
Everything Plants
Soltech Solutions 15% discount code 'everythingplants' https://glnk.io/0lyv/9 VeryPlants use the code EverythingPlants10 for 10% ...
true
16:40
Now playing
Tour of House Plant in their Natural Environment (outdoors!)
57K 5y ago
Plant Vibrations with Devin Wallien
In this longer video I take a walking tour of the neighborhood El Poblado in Medellin, Colombia showing many of the most popular ...
14:08 there's really nothing like it and then let's finish with many of our favorite plant the beautiful monstera deliciosa this plant has ...
CC
Shorts
The MOST DANGEROUS Fruit? - The Monstera Deliciosa Fruit!
229K views
The best way to get your monstera leaves to grow is to attach them to something for support!
2.9M views
Trying Monstera Deliciosa fruit and growing seeds! #monsteraplant #growingseeds
315K views
Giant Monstera! #plants #monsteradeliciosa
18K views
The 9 Monstera Deliciosa fruits are getting bigger and will soon reach maturity 💚 #monsteradeliciosa
64K views
true
10:49
Now playing
You’re (Probably) Killing Your Monstera
374K 3y ago
Sheffield Made Plants
You're probably killing your Monstera if you make this common mistake. -------------------- Download my FREE Plant Parent's ...
4K
Summary
Sheffield Made Plants explores why pruning monsteras won't create a bushier appearance like other plants. Instead, learn how to properly propagate and stake these climbers to achieve a lush look without sacrificing mature foliage.
true
41:52
Now playing
How to continuously grow a large Monstera
146K 1y ago
Sydney Plant Guy
I am chopping my Monstera adansonii in half & re-extend both: one with Moss to grow indoors, one with aroid mix to grow ...
4K
Summary
Sydney Plant Guy demonstrates the 'chop and extend' process to manage a large Monstera adansonii in an indoor setting. The video covers techniques for maintaining plant stability using moss poles and garden stakes while preparing for future growth.
true
23:55
Now playing
How To Shop & Care For MONSTERA DELICIOSA - Plant Care Light, Repot, Soil, Water Houseplant Care 101
80K 2y ago
Plant Life with Ashley Anita
monstera #monsteradeliciosa #monsteraplant Visit https://bit.ly/3rRx8hV and use code ASHLEY at checkout to get 10% off any ...
Summary
Ashley Anita guides viewers through the entire process of acquiring a new Monstera Deliciosa, from assessing plant health and pest checks at the nursery to the final potting steps. The episode covers selecting the right pot size, identifying when to repot, and creating the ideal, well-draining soil mix for optimal growth.
true
2:29
Now playing
How To Grow ~ Monstera
5.6K 3y ago
Urban Abundance
Also known as the "Swiss cheese plant", monstera deliciosa is a tropical plant that is often used as a house plant because it is so ...
0:00 Hey what's up guys it's farmer Jack and today we're going to talk about monstera deliciosa. So this plant is also known as The ...
true
6:41
Now playing
Grow Monstera From Seed ( 0 - 8 Months )
89K 4y ago
Techplant
In this video I show you how to start monstera seeds and grow them into small plants! Growing Monstera deliciosa from seed is ...
4K
true
23:26
Now playing
Houseplant Care 101: Monstera Care Tips - Monstera Watering, Feeding, & Repotting
17K 4y ago
Plant Life with Ashley Anita
houseplants #monstera #plants Visit https://bit.ly/3rRx8hV and use code ASHLEY at checkout to get 10% off any potting mix.
0:44 ... when it comes to caring for your monstera deliciosa now on these three topics i've already done very very detailed episodes i've ...
true
10:01
Now playing
How To Propagate MONSTERA DELICIOSA 2 EASY Ways (DETAILED)
1.7M 6y ago
Techplant
HowTo #Propagate #MonsteraDeliciosa How To Propagate Monstrea Deliciosa Cuttings Easily! I tried using two different methods ...
4K
6 chapters
Intro | Cutting | Water Cutting | Spagna Moss Cutting | Mother Plant | Outro
true
8:13
Now playing
Easy IKEA Jungle Box (DIY Greenhouse)
1.7M 2y ago
SerpaDesign
I've made plenty of IKEA projects before, and we thought it was time for another. Well, actually, it was my wife's idea. A few months ...
4K
Summary
SerpaDesign transforms a standard IKEA cabinet into a self-sustaining greenhouse for lush tropical plants. The build involves custom glass sealing, a cork-panelled background, and specific drainage features to support thriving foliage like Alocasia and various bromeliads.
true
35:28
Now playing
All you need to know about Monstera Deliciosa
610K 7y ago
Swedish Plantguys
We hope you will enjoy this video! And thank you for watching ! Chapters: Purchase start of the video Planting 08:14 Placement ...
true
12:47
Now playing
#91 Top 6 Common Indoor Plants That Can Grow In Water | Grow Houseplants Without Soil
11M 5y ago
The Green Earth
My top 6 Indoor Plants that are growing in just water without soil. Growing plants in water, whether houseplants or an indoor herb ...
2 products
4K
12 chapters
Introduction | Cutting of Satin Pothos | Cutting of Golden Pothos | Cutting of Jade Plant | Cutting of ZZ Plant | Cutting of Rubber Plant | Cutting of Snake Plant | Day 01 of Water Propagation | Day 20 of Water Propagation | Day 34 of Water Propagation | Day 45 of Water Propagation | Growing Indoor Plants in LECA
true
10:47
Now playing
I Stopped Killing Jade Plants Once I Knew This
562K 1y ago
Sheffield Made Plants
Download my FREE Plant Parent's Troubleshooting Handbook https://resources.sheffieldmadeplants.com/handbook ...
4K
Summary
Sheffield Made Plants shares techniques for cultivating healthy jade plants, emphasizing the importance of ample sunlight and proper watering habits. Learn how to encourage a bushier structure through strategic pruning and explore methods for propagating new plants from cuttings.
true
1:41:19
Now playing
The ULTIMATE Guide To The 10 Most Popular Plants
258K 2y ago
Sheffield Made Plants
Download my FREE Plant Parent's Troubleshooting Handbook https://resources.sheffieldmadeplants.com/handbook ▸ Book a ...
4K
true
21:20
Now playing
Houseplants I would NEVER buy again! (and some that I would)
54K 1y ago
Kaylee Ellen
Some houseplants just aren't worth it… but a few totally are! Here are the ones I'd buy again. Anthurium Rare Plant Index: ...
Summary
Kaylee Ellen shares her personal experiences with specific plant varieties, explaining why some species prove to be too high-maintenance for a relaxing collection. The discussion covers growth patterns, light requirements, and the effort needed to keep these popular indoor plants looking their best over time.
true
16:09
Now playing
Making Thick Trunk Bonsai from Jade Plant | in 2 Years | Pruning | Repotting | Portulacaria Afra
3.6M 3y ago
What a Plant
Now You can Buy My Premium Bonsai Soil Mix (From Link Below) https://amzn.to/358xd84 Appreciate Your Support Please ...
13 chapters
Intro | Pruning | Preparing Bonsai Pot | Repotting | 2nd Pruning | 1st Result | Repotting into Bigger Pot | 3rd Pruning | Preparing Bonsai Pot | Repotting | Growth After Few Months | 4th Pruning | Final Result
true
17:04
Now playing
AFTER I CREATED THIS METHOD I NEVER LOST AN ORCHIDS AGAIN
2.1M 1y ago
Wonder Garden
Welcome to Wonder Garden! In today's video, discover the ultimate game-changing method that ensures you'll never lose an ...
3 products
4K
Shorts
😲 8FT TALL Monstera Deliciosa Repot 🌿 PART 1 #monstera #houseplants #indoorplants #repotting #plants
13K views
Monstera Deliciosa Aurea Variegated 🪴✨🌙 #plants #plantlover #indoorplants #monstera #houseplants
7.3K views
New
She’s thriving ✨😍 #monstera #monsteradeliciosa #giantmonstera #plants #houseplants
961 views
New
5 Poisonous Fruits You Should Never Eat Unripe! ⚠️Monstera Deliciosa.
7.7K views
New
1 year Monstera Bulbasaur TC update. #monstera #philodendron #plantcare #anthurium #plantcaretips
19K views
true
18:21
Now playing
18 Gorgeous Houseplants That Thrive Without Sunlight
839K 1y ago
Natural Health Remedies
18 Gorgeous Houseplants That Thrive Without Sunlight Don't have a sunny window? No problem! In this video, we're showing ...
Summary
Natural Health Remedies explores diverse options for dimly lit spaces, from vining plants to flowering varieties. Learn specific care tips for each plant, including watering frequency and soil preferences, to ensure longevity without direct sunlight.
true
25:13
Now playing
Just one slice of onion can make Anthurium explode with so many beautiful flowers
542K 1y ago
Wonder Garden
Welcome to Wonder Garden! In today's video, discover an astonishing trick: just one slice of onion can make your Anthurium ...
2 products
4K
true
3:38
Now playing
STOP doing this to your Monstera Deliciosa
349K 4y ago
Kill This Plant
I get the idea behind it. But this is not doing what you think its doing. We need to stop pruning our Monstera Deliciosa. Pruning this ...
4K
Summary
Kill This Plant explains how pruning affects hormone pathways and growth in Monsteras, unlike branching plants like jade. They reveal how cutting stems reverts plants to a juvenile state with smaller leaves and fenestrations.
true
14:42
Now playing
The only 4 Monstera Deliciosa tips you need to know
91K 5mo ago
Kill This Plant
Download My Houseplant App - https://blackthumbplant.app/ Join Our Discord for plant help - https://discord.gg/kw9FnABCRk ...
4K
5 chapters
Find the back of the plant | Measure your light | Get grow lights | Take top cuttings | Conclusion
true
11:01
Now playing
Tips + Tricks For Monstera Deliciosa | ALL YOU NEED TO KNOW 🌿 Indoor Plant Tips For Beginners
47K 5y ago
The Jungle Haven
Monstera deliciosa care tips - swiss cheese plant care - monstera houseplant tips and tricks - indoor plant tips for beginners ...
true
41:52
Now playing
How to continuously grow a large Monstera
146K 1y ago
Sydney Plant Guy
I am chopping my Monstera adansonii in half & re-extend both: one with Moss to grow indoors, one with aroid mix to grow ...
4K
Summary
Sydney Plant Guy demonstrates the 'chop and extend' process to manage a large Monstera adansonii in an indoor setting. The video covers techniques for maintaining plant stability using moss poles and garden stakes while preparing for future growth.
Shorts
5 years of monstera growth! #monstera #plants
696K views
Repotting my NEW indoor monstera plant🪴 #plants #monstera #indoorplants #houseplants #monsteraplant
315K views
New
Converting my M. deliciosa ‘brazil form’ from grown on wood to grown on a moss pole
26K views
Deliciosa VS Borsigiana Monstera | They’re VERY Different - Which One is Yours?
203K views
I have 600 houseplants and my Monstera Deliciosa my absolute FAVORITE plant in my collection 🤍🪴🥹
83K views
true
5:46
Now playing
How to Grow Monstera Deliciosa
6.5K 3mo ago
Tanner The Planter
Day 21/30 of teaching you how to grow your houseplants in a simple way. My reference guide to growing 150 of the most popular ...
Summary
Tanner The Planter provides a comprehensive guide on Monstera deliciosa care, focusing on proper soil composition, effective watering techniques, and optimal lighting conditions. Learn how to encourage healthy root systems and leaf maturity while managing growth through staking.
true
10:15
Now playing
cutting down my GIANT Monstera Deliciosa / Taking Cuttings / Repotting / Feed & Care Nicole's Garden
80K 2y ago
Nicole’s Garden
Please Like and Subscribe! Affiliate links Ponytail sunhat (CAN) https://amzn.to/3UKGeeG Hori Hori Knife (CAN) ...
true
17:59
Now playing
Proper care for Monstera deliciosa! These 7 care tips are a MUST-know 🪴 (Swiss cheese plant)
40K 9mo ago
crazyplantboy
Want your Monstera deliciosa to grow better and healthier? Need an overview of the most important care tips so you can ...
4K
true
2:42
Now playing
239 Days Monstera Deliciosa Time-Lapse – From Seed to Stunning Plant! 🌱
83K 1y ago
Interesting as FCK
Ever wondered how long it really takes to grow a Monstera deliciosa from seed? Here's the full 239-day journey captured in one ...
4K
true
2:52
Now playing
How to Repot Monstera Deliciosa. IT'S EASY!
149K 3y ago
Tanner The Planter
In this video we discuss and demonstrate the general methods of properly re-potting the Monstera Deliciosa. Find Your PERFECT ...
5 chapters
Intro | Preparing the soil | Checking the roots | Repot | Watering
true
19:41
Now playing
Monstera Deliciosa: From Jungle Survivor to Icon 🌿
19K 1y ago
Plantrums
Watch next: The Orchid's Dark Past — Obsession, Mystery, and Survival → https://www.youtube.com/watch?v=q7VpOcKEpAw ...
4 products
7 chapters
Intro | Monstera’s Ancient History | Monstera’s Early Documentation | Monstera’s Biological Secrets | Monstera’s Rise to Fame | Monstera’s Ecological Impact | Monstera’s True Value
Shorts
Eating Monstera Deliciosa Fruit, Finally!
471K views
What to do with your monstera aerial roots? #monsteraplant #plantcaretips #plantcare
981K views
New
Monstera deliciosa (exotic plant that produces fruit) #shorts  #foodforest #tropical  #gardening
1.1K views
A lot of growth already in a year and a half for big mama Monstera Deliciosa. #monsteraplant
433K views
Monstera Deliciosa Repot #monstera #monsteraplant #repotting
49K views
true
11:25
Now playing
How to start Monstera the right way
61K 4mo ago
Kill This Plant
Download My Houseplant App - https://blackthumbplant.app/ Join Our Discord for plant help - https://discord.gg/kw9FnABCRk ...
4K
Summary
Kill This Plant provides a comprehensive guide to maximizing growth for single-stem Monsteras. The discussion covers essential care routines, including light intensity monitoring, selecting appropriate soil amendments to improve drainage, proper staking techniques, and safe fertilizing schedules to help plants thrive.
true
5:37
Now playing
How to Add Support & Prune Back Overgrown Monstera Deliciosa Plants
242K 3y ago
My City Plants
How to prune Monstera Deliciosa (complete guide) ...
5 products
Earns commission
Summary
My City Plants demonstrates techniques for managing large foliage using bamboo stakes, raffia, and clean pruners. Learn how to secure sprawling stems for a narrower profile and selectively cut back growth to maintain a balanced shape.
true
7:13
Now playing
You don’t need to repot your Monstera
80K 2y ago
Kill This Plant
Join our discord community https://discord.gg/RYXzghkuhk * Get 15% off Soltech Grow Lights with promo code: KTP2025 ...
4K
Summary
Kill This Plant explains how to determine when a Monstera actually needs a larger container based on root volume and soil moisture. Learn to identify the right balance of root mass to soil, and discover alternative solutions for managing water retention and light requirements.
true
19:37
Now playing
Saving a Giant: I got an 11+ year old Monstera Deliciosa Large Form
210K 3y ago
Leafy_Luster
MONSTERA DELICIOSA large form exists & I finally found one to call my own. Join me on this Massive Monstera Makeover to ...
4K
true
12:31
Now playing
The Full Story of my 11+ year old Monstera Deliciosa Large Form
108K 2y ago
Leafy_Luster
It's one of my most requested videos to make. THE UPDATE on my rescue Monstera Deliciosa! My rescue mission was quite ...
4K
Shorts
Repotting 7ft+ monstera #shorts #gardenup #monstera #plants
409K views
Monstera Fruit: The Strange Tropical Fruit 🍃🟢
102K views
let's propagate monstera #monstera #propagate #plants #soil #water #roots #areal #cutting #plants
1M views
Grow a BIG Monstera✨ #houseplants #plantcare #plantcaretips #monsteradeliciosa
3.7K views
If you love your monstera, set it free #houseplant #monstera #plantgrowth
351K views
true
14:33
Now playing
MONSTERA PROPAGATION: How to Propagate Monstera deliciosa (2023)
153K 4y ago
Plant Vibrations with Devin Wallien
My favorite Houseplant tools here: https://amzn.to/3sYJ3ta Yesssss, finally I have created a video on how to successfully ...
CC
true
5:00
Now playing
5 quick tips Monstera Deliciosa
88K 5y ago
Swedish Plantguys
In this video we give you 5 quick tips for your Monstera Deliciosa also know as Swiss cheese plant. Enjoy! And thank you for ...
4K
7 chapters
Intro | Juvenile Monstera | Repot | Support | Prune | Humidity | Outro
true
14:29
Now playing
How to Repot a Monstera Deliciosa Plant | Staking | Aerial Roots | Care Guide
37K 3y ago
Ana Perez
Hello friends, here's how to repot a monstera deliciosa plant and how to care for a monstera plant as well! Here are all the ...
4K
CC
12 chapters
intro | When to repot a monstera deliciosa? | What to do when a monstera is really root bound? | What type of potting soil to use for a monstera deliciosa? | What type of pot to use for a monstera? | Are moss poles necessary for a monstera? | What to do with the aerial roots of a monstera | What type of light does a monstera need? | How often should you water a monstera? | How much water should you give a monstera plant? | Fertilizing a monstera plant | Pest Preventions
true
17:27
Now playing
Monstera Deliciosa Single Node Propagation (0-9 Months)
609K 4y ago
Techplant
propagating monstera nodes. In this video I will show you the process of making single node cuttings of monstera deliciosa and ...
1 product
4K
true
9:13
Now playing
5 Things Monsteras HATE That Might Surprise You
387K 3y ago
Sheffield Made Plants
Monsteras aren't particularly fussy plants but there are some key things they hate that might actually surprise you.
Summary
Sheffield Made Plants explains how common care habits like misting, incorrect potting, and allowing dust buildup can harm Monsteras. Learn about the importance of stable humidity, proper root support, and leaf hygiene for optimal plant health and leaf development.
true
11:47
Now playing
MONSTERA DELICIOSA Care | Swiss Cheese Plant Tips & Tricks
187K 7y ago
Good Growing
For bonus content, subscribe to my Patreon ›› https://www.patreon.com/goodgrowing Do you like being trendy? Want plants to ...
11 chapters
Intro | General Info | Water | Light | Temperature | Humidity | Fertiliser | Propagation | Toxicity | Growth Care | Outro
Shorts
Monstera Propagation (My Step by Step Process)
313K views
I CHOPPED UP my Monstera to get it to thrive #houseplants #plants #houseplant #monstera
804K views
Monstera Deliciosa 🌿 Complete Care, Propagation, Height & Fruit Facts!#youtubeshorts#shorts#reels
2.2K views
New
Propagation Loop♻️🌿 #planttips #plants #monstera #propagation #plantcare #diy
18K views
Monstera Deliciosa 2023 Season is Here
1.8M views
true
10:39
Now playing
MONSTERA DELICIOSA CARE + PROPAGATION | indoor swiss cheese plant care
174K 6y ago
Good & Planty
this MONSTER of a care guide is here monstera leaves going yellow? not getting those fenestrations? is your swiss cheese ...
CC
6 chapters
Intro | Water | Light | Repotting | Temperature Humidity | Propagation
true
10:25
Now playing
Monstera richtig pflegen – der komplette Guide | feey 🌱
129K 3y ago
feey
In diesem Video zeigen wir dir alles, was du zur richtigen Pflege deiner Monstera wissen musst: Dazu zählt der richtige Standort, ...
2:46 Das hier ist der Klassiker der Monsteras: die Monstera deliciosa. Diese findest du bei uns im Shop in ganz vielen verschiedenen ...
4K
CC
true
7:35
Now playing
Watch this before you separate your Monstera
38K 1y ago
Kill This Plant
Get 15% off Soltech Grow Lights with promo code: KTP2025 https://glnk.io/0lyv/13 * GET 10% off the best professional ...
4K
Summary
Kill This Plant explains how to properly assess indoor lighting and choose the correct pot size to prevent root rot after propagating. Using a light meter and moisture probe, they demonstrate the relationship between soil volume, oxygen levels, and plant stress.
true
35:40
Now playing
My Monstera Jungle Mint Started Reverting...So I Chopped It All Back 👀
16K 6mo ago
plantsbymelissa
My Monstera jungle mint started reverting, so it was time to cut it back and restart. In this video I'm chopping it down, resetting the ...
4K
8 chapters
Overview | Time to chop | Repotting the base | Chunky pon | Watering | Cuttings | Back in my plant room | Outro
true
15:38
Now playing
Every Type of Monstera Deliciosa Explained (And Which One to Buy)
85K 2y ago
Kaylee Ellen
M E R C H : https://kayleeellen.creator-spring.com/? Monstera Vareities: ...
Summary
Kaylee Ellen outlines the differences between small and large form Monstera deliciosa, focusing on internodal spacing, growth patterns, and leaf characteristics. Learn which variety best suits your available space and aesthetic preferences through a detailed comparison of popular green and variegated cultivars.
Shorts
The MOST DANGEROUS Fruit? - The Monstera Deliciosa Fruit!
229K views
MONSTERA Propagation with 100% SUCCESS | Watch Me Turn a sad 😢 Plant into 11 Happy 🤓babies!
607K views
Monstera Deliciosa : The Delicious Monster Houseplant ! 🪴🍍#garden  #farming #organic
3.1M views
Monstera deliciosa fruit #asmr
901K views
Would you do this?! Full Monstera Cut Back!
456K views
true
9:13
Now playing
5 Things Monsteras HATE That Might Surprise You
387K 3y ago
Sheffield Made Plants
Monsteras aren't particularly fussy plants but there are some key things they hate that might actually surprise you.
Summary
Sheffield Made Plants explains how common care habits like misting, incorrect potting, and allowing dust buildup can harm Monsteras. Learn about the importance of stable humidity, proper root support, and leaf hygiene for optimal plant health and leaf development.
true
11:00
Now playing
How to water propagate a Monstera Deliciosa
64K 1y ago
Kill This Plant
Download My Houseplant App - https://blackthumbplant.app/ Join Our Discord for plant help - https://discord.gg/kw9FnABCRk ...
4K
Summary
Kill This Plant demonstrates how to take top and mid cuttings for propagation while ensuring healthy root development and foliage maintenance. Learn techniques to prevent rot, choose appropriate pot sizes based on root mass, and mix suitable soil for growth.
Shorts
Monstera deliciosa harvest!
4.4M views
Here’s 4 tips to grow a HUGE Monstera Deliciosa! 🌱🪴
286K views
The Secret, Edible Fruit Hiding on Your Houseplant #monstera #animation
713K views
Monstera deliciosa propagation in water❤️#shorts #youtubeshorts #like #viral #fyp
425K views
Monstera Deliciosa Fruit #shorts
51K views
true
2:41
Now playing
My Mutant Monstera Deliciosa Update
7.9K 2y ago
Techplant
Amazon Affiliate link to light https://amzn.to/3S7wJVN These are amazon affiliate links, they don't change the price, however after ...
4K
true
0:47
Now playing
Monstera Deliciosa Time Lapse (HUGE LEAF)
28K 3y ago
Interesting as FCK
In this plant time lapse, we'll be taking a look at a GIANT monstera deliciosa growing, one of my favorite plants in the world!
4K
true
8:16
Now playing
Monstera Deliciosa LARGE FORM: Repotting & Root Dividing
136K 6y ago
reallybigplant
How to repot an extra large Monstera Deliciosa with a huge developed root system! Join me while I try to root divide this Monstera ...
true
8:16
Now playing
How to Grow and Care for Monstera (Swiss Cheese) Indoors
615K 5y ago
Garden Up
LINKS| Good quality soil : https://gardenup.in/products/ready-potting-soil-mix Succulent soil ...
true
7:41
Now playing
Monstera Roots - 4 Things to do with Aerial Roots - Monstera deliciosa (2023)
189K 3y ago
Plant Vibrations with Devin Wallien
Shop plants here! https://plantvibrations-shop.myshopify.com/ While we're growing our Monstera deliciosa or many other aroid ...
CC
Summary
Devin Wallien explains how aerial roots function as nutrient and water absorbers for the plant. The video demonstrates practical techniques for managing these roots, including guiding them into a moss pole, burying them in the soil, or placing them in water to enhance growth.
true
12:09
Now playing
The Easiest Monsteras to Care For 🌿 Great for Beginners Too!
3K 3mo ago
planty skies
🌿 Monsteras are among the most popular houseplants – but which varieties are truly easy to care for and suitable for beginners ...
4K
true
6:11
Now playing
The Giant Monstera Deliciosa Node Update
12K 1y ago
Techplant
Here are the giant monstera nodes I've had propagating for 2 yeara These are amazon affiliate links, they don't change the price, ...
4K
Summary
Techplant documents a two-year propagation experiment using various methods to revive severely rotted monstera cuttings. Despite initial setbaks and neglect, the video tracks the long-term survival and growth of the nodes under different environmental c
Sign in
Home
Shorts
Subscriptions
You
History
Sign in to like videos, comment, and subscribe.
Sign in
Explore
Music
Movies & TV
Live
Show more
More from YouTube

  
    
    
  
YouTube Premium

  
    
    
    
  
YouTube Music

  
    
      
      
    
    
      
      
    
  
YouTube Kids
Report history
AboutPressCopyrightContact usCreatorsAdvertiseDevelopersImpressumCancel Memberships
TermsPrivacyPolicy & SafetyHow YouTube worksTest new features
© 2026 Google LLC
All
Shorts
Unwatched
Watched
Videos
Recently uploaded
Live
About these results
Filters
Shorts
Step 1 for Monstera deliciosa: The BIG chop to propagate #monsteraplant #plantpropagation
1.6M views
Monstera deliciosa - Thai Constellation
30K views
This is how I grew my monstera deliciosa from seed! #timelapse #monstera
767K views
Never underestimate a well grown classic #monsteradeliciosa grown by @plantsdecoranddiy
299K views
The MOST DANGEROUS Fruit? - The Monstera Deliciosa Fruit!
229K views
true
11:21
Now playing
If I Only Knew These Monstera Tips 5 Years Ago
2M 3y ago
Sheffield Made Plants
My best Monstera care tips I wish I knew 5 years ago. -------------------- Download my FREE Plant Parent's Troubleshooting ...
9:12 ... actually two types of monstera plants you can buy that look very similar the monstera bozigiana and the monstera deliciosa they ...
4K
true
4:28
Now playing
Do this after you buy your first Monstera
204K 2y ago
Kill This Plant
Download My Houseplant App - https://blackthumbplant.app/ Join Our Discord for plant help - https://discord.gg/kw9FnABCRk ...
4K
Summary
Kill This Plant analyzes why a new Monstera may struggle by measuring light levels and soil moisture. Discover how to identify if a plant is receiving enough light and why adjusting the environment immediately after purchase is crucial for healthy growth.
true
8:12
Now playing
How to grow the BEST Monstera
1.3M 2y ago
Kill This Plant
Download My Houseplant App - https://blackthumbplant.app/ Join Our Discord for plant help - https://discord.gg/kw9FnABCRk ...
1 product
4K
Summary
Kill This Plant compares top cuttings versus rearranging existing vines to fix crawling Monsteras. Learn methods for staking, pruning for propagation, and managing soil requirements for healthier, upright growth.
true
4:06
Now playing
how to grow a monstera as fast as possible
57K 9mo ago
Kill This Plant
Download My Houseplant App - https://blackthumbplant.app/ Join Our Discord for plant help - https://discord.gg/kw9FnABCRk ...
4K
Summary
Kill This Plant explains how to optimize monstera growth by mastering water saturation, soil drainage mixtures, and light intensity. Learn to use tools for testing soil moisture and tips for extending daylight hours to ensure consistent, rapid plant development.
true
8:01
Now playing
7 MONSTERA Plant Care Tips You Need to KNOW : Monstera Deliciousa
340K 3y ago
MonstroFarm
I want to help you grow beautiful Monstera's, which is Super-Easy. Here are 7 Plant Care Tips on how I care and make sure that ...
8 chapters
Intro | LIGHTING | TEMPERATURE AND HUMIDITY | SOIL MIXTURE | FERTILIZER | PRUNING | PEST MANAGEMENT | WATERING
true
2:42
Now playing
239 Days Monstera Deliciosa Time-Lapse – From Seed to Stunning Plant! 🌱
83K 1y ago
Interesting as FCK
Ever wondered how long it really takes to grow a Monstera deliciosa from seed? Here's the full 239-day journey captured in one ...
4K
true
10:22
Now playing
0:19
10:22
Set Up Your Monstera For Success
263K 2y ago
Sheffield Made Plants
Download my FREE Plant Parent's Troubleshooting Handbook https://resources.sheffieldmadeplants.com/handbook ▸ 1:1 ...
4K
Matching chapter
0:19 My Monstera
true
3:13
Now playing
why people keep Killing their Monstera
1M 3y ago
Kill This Plant
Hopefully this video will get you to stop killing your Monstera Deliciosa. *Links in this description contain affiliate links.
0:00 Monstera deliciosa is supposed to be one of the easiest plants but you would not believe how many people are asking for help but ...
4K
true
5:18
Now playing
Why you need to cut your monstera
337K 2y ago
Kill This Plant
Download My Houseplant App - https://blackthumbplant.app/ Join Our Discord for plant help - https://discord.gg/kw9FnABCRk ...
4K
Summary
Kill This Plant demonstrates the process of air layering a large Monstera to safely propagate new roots before cutting. This technique allows for maintaining large foliage while preparing the plant for a fresh start in a new pot.
true
49:16
Now playing
I Wish I Knew These Monstera Tips 5 Years Ago
562K 2y ago
Sheffield Made Plants
All my best Monstera tips in 1 video! -------------------- Download my FREE Plant Parent's Troubleshooting Handbook ...
0:00 Perforations or holes in the Le or monstera deliciosa is one of the most highly Sal offer Commodities in the world sort of if you ...
4K
true
7:22
Now playing
Do These 3 Things For The Largest Monstera Deliciosa
70K 3y ago
Techplant
How to grow monstera deliciosa the right way for large leaves. These tips will help you grow a really nice monstera deliciosa!
4K
Summary
Techplant clarifies common misconceptions about pruning lower leaves and recommends single-stalk configurations for easier management. Discover the importance of proper orientation when securing plants to supports, ensuring the aerial roots are pressed against the pole for maximum leaf growth.
true
12:30
Now playing
The World’s Largest Monstera Deliciosa Plant
62K 3y ago
SleepyLizard
Get your Monstera Delciosa fruit and avocados at: https://www.guacfarm.com.
Summary
SleepyLizard showcases a massive Monstera Deliciosa plantation focused on fruit production rather than decoration. The video explores harvesting the sweet fruit and the agricultural challenges of integrating these plants within an avocado grove.
true
11:06
Now playing
10 Must Do Tricks For The BEST Monstera
714K 3y ago
Sheffield Made Plants
Head to https://squarespace.com/sheffield to save 10% off your first purchase of a website or domain using code SHEFFIELD.
4K
true
10:59
Now playing
How To Propagate Your Monstera The Right Way
347K 2y ago
Sheffield Made Plants
Monstera propagation the right way -------------------- Download my FREE Plant Parent's Troubleshooting Handbook ...
0:00 This is The Ultimate Guide to propagating and multiplying your monstera deliciosa or swiss cheese plant so you can have more of ...
4K
true
16:00
Now playing
Why You shouldn’t buy a Monstera Deliciosa !
15K 1y ago
The Millennial Plant Dad
The Monstera Deliciosa may be an iconic house plant staple but the fact of the matter is that it's not for everyone, home or space !
4K
7 chapters
Intro | Space | How it is sold | M poles | Climbing | Fertilizing | Outro
true
7:34
Now playing
Eating The Tastiest Fruit In The World! - How to Eat Monstera Deliciosa Safely | Mexican Breadfruit
657K 3y ago
Greenthumbs Garden
Monstera Delcicosa is a popular houseplant all over the world. What many do not know is that when grown outside in the right ...
CC
6 chapters
The Monstera Delcicosa Fruit | Monstera Delcicosa Fruit Nutritional Benefits | When To Harvest Monstera Delcicosa Fruit | When Is It Safe To Eat Monstera Delcicosa Fruit | What To Do With Unripe Monstera Delcicosa Fruit | Tasting The Monstera Delcicosa Fruit
true
11:05
Now playing
Grow Your Pothos 3x FASTER
768K 2y ago
Sheffield Made Plants
Download my FREE Plant Parent's Troubleshooting Handbook https://resources.sheffieldmadeplants.com/handbook ▸ 1:1 ...
4K
Summary
Sheffield Made Plants shares techniques for maximizing pothos growth through strategic potting, stem pinning, and propagation methods. Learn how proper lighting, soil selection, and understanding root health contribute to fuller, faster-growing vines.
true
12:48
Now playing
Popular Plant Products I’ll NEVER Buy Again (And What I Use Instead)
272K 1y ago
Sheffield Made Plants
Download my FREE Plant Parent's Troubleshooting Handbook https://resources.sheffieldmadeplants.com/handbook ...
4K
10 chapters
Intro | Rooting Hormone | Miracle Grow | Clear Pots | Decorative pots | Grow lights | Velcro plant eyes | Leaf shine | Core poles | Scissors
true
44:26
Now playing
Inside an Indoor Plant Oasis | Full Tour
325K 1y ago
Sheffield Made Plants
Download my FREE Plant Parent's Troubleshooting Handbook https://resources.sheffieldmadeplants.com/handbook ...
4K
Summary
Richard provides a comprehensive look into a living space transformed into a dense indoor jungle featuring over 200 diverse house plants. The tour showcases popular species alongside unique finds, sharing propagation tips and creative styling methods for various leaf types.
true
10:21
Now playing
Changing THIS Will Make Your Spider Plant THRIVE
914K 1y ago
Sheffield Made Plants
Download my FREE Plant Parent's Troubleshooting Handbook https://resources.sheffieldmadeplants.com/handbook ...
4K
6 chapters
Intro | Leaf Problems | Why Does This Work | Fertilizer | pH | Water Conditioner
true
12:26
Now playing
Growing A MATURE MONSTERA | How Long Does It Take???
26K 3y ago
Everything Plants
Soltech Solutions 15% discount code 'everythingplants' https://glnk.io/0lyv/9 VeryPlants use the code EverythingPlants10 for 10% ...
true
16:40
Now playing
Tour of House Plant in their Natural Environment (outdoors!)
57K 5y ago
Plant Vibrations with Devin Wallien
In this longer video I take a walking tour of the neighborhood El Poblado in Medellin, Colombia showing many of the most popular ...
14:08 there's really nothing like it and then let's finish with many of our favorite plant the beautiful monstera deliciosa this plant has ...
CC
Shorts
The MOST DANGEROUS Fruit? - The Monstera Deliciosa Fruit!
229K views
The best way to get your monstera leaves to grow is to attach them to something for support!
2.9M views
Trying Monstera Deliciosa fruit and growing seeds! #monsteraplant #growingseeds
315K views
Giant Monstera! #plants #monsteradeliciosa
18K views
The 9 Monstera Deliciosa fruits are getting bigger and will soon reach maturity 💚 #monsteradeliciosa
64K views
true
10:49
Now playing
You’re (Probably) Killing Your Monstera
374K 3y ago
Sheffield Made Plants
You're probably killing your Monstera if you make this common mistake. -------------------- Download my FREE Plant Parent's ...
4K
Summary
Sheffield Made Plants explores why pruning monsteras won't create a bushier appearance like other plants. Instead, learn how to properly propagate and stake these climbers to achieve a lush look without sacrificing mature foliage.
true
41:52
Now playing
How to continuously grow a large Monstera
146K 1y ago
Sydney Plant Guy
I am chopping my Monstera adansonii in half & re-extend both: one with Moss to grow indoors, one with aroid mix to grow ...
4K
Summary
Sydney Plant Guy demonstrates the 'chop and extend' process to manage a large Monstera adansonii in an indoor setting. The video covers techniques for maintaining plant stability using moss poles and garden stakes while preparing for future growth.
true
23:55
Now playing
How To Shop & Care For MONSTERA DELICIOSA - Plant Care Light, Repot, Soil, Water Houseplant Care 101
80K 2y ago
Plant Life with Ashley Anita
monstera #monsteradeliciosa #monsteraplant Visit https://bit.ly/3rRx8hV and use code ASHLEY at checkout to get 10% off any ...
Summary
Ashley Anita guides viewers through the entire process of acquiring a new Monstera Deliciosa, from assessing plant health and pest checks at the nursery to the final potting steps. The episode covers selecting the right pot size, identifying when to repot, and creating the ideal, well-draining soil mix for optimal growth.
true
2:29
Now playing
How To Grow ~ Monstera
5.6K 3y ago
Urban Abundance
Also known as the "Swiss cheese plant", monstera deliciosa is a tropical plant that is often used as a house plant because it is so ...
0:00 Hey what's up guys it's farmer Jack and today we're going to talk about monstera deliciosa. So this plant is also known as The ...
true
6:41
Now playing
Grow Monstera From Seed ( 0 - 8 Months )
89K 4y ago
Techplant
In this video I show you how to start monstera seeds and grow them into small plants! Growing Monstera deliciosa from seed is ...
4K
true
23:26
Now playing
Houseplant Care 101: Monstera Care Tips - Monstera Watering, Feeding, & Repotting
17K 4y ago
Plant Life with Ashley Anita
houseplants #monstera #plants Visit https://bit.ly/3rRx8hV and use code ASHLEY at checkout to get 10% off any potting mix.
0:44 ... when it comes to caring for your monstera deliciosa now on these three topics i've already done very very detailed episodes i've ...
true
10:01
Now playing
How To Propagate MONSTERA DELICIOSA 2 EASY Ways (DETAILED)
1.7M 6y ago
Techplant
HowTo #Propagate #MonsteraDeliciosa How To Propagate Monstrea Deliciosa Cuttings Easily! I tried using two different methods ...
4K
6 chapters
Intro | Cutting | Water Cutting | Spagna Moss Cutting | Mother Plant | Outro
true
8:13
Now playing
Easy IKEA Jungle Box (DIY Greenhouse)
1.7M 2y ago
SerpaDesign
I've made plenty of IKEA projects before, and we thought it was time for another. Well, actually, it was my wife's idea. A few months ...
4K
Summary
SerpaDesign transforms a standard IKEA cabinet into a self-sustaining greenhouse for lush tropical plants. The build involves custom glass sealing, a cork-panelled background, and specific drainage features to support thriving foliage like Alocasia and various bromeliads.
true
35:28
Now playing
All you need to know about Monstera Deliciosa
610K 7y ago
Swedish Plantguys
We hope you will enjoy this video! And thank you for watching ! Chapters: Purchase start of the video Planting 08:14 Placement ...
true
12:47
Now playing
#91 Top 6 Common Indoor Plants That Can Grow In Water | Grow Houseplants Without Soil
11M 5y ago
The Green Earth
My top 6 Indoor Plants that are growing in just water without soil. Growing plants in water, whether houseplants or an indoor herb ...
2 products
4K
12 chapters
Introduction | Cutting of Satin Pothos | Cutting of Golden Pothos | Cutting of Jade Plant | Cutting of ZZ Plant | Cutting of Rubber Plant | Cutting of Snake Plant | Day 01 of Water Propagation | Day 20 of Water Propagation | Day 34 of Water Propagation | Day 45 of Water Propagation | Growing Indoor Plants in LECA
true
10:47
Now playing
I Stopped Killing Jade Plants Once I Knew This
562K 1y ago
Sheffield Made Plants
Download my FREE Plant Parent's Troubleshooting Handbook https://resources.sheffieldmadeplants.com/handbook ...
4K
Summary
Sheffield Made Plants shares techniques for cultivating healthy jade plants, emphasizing the importance of ample sunlight and proper watering habits. Learn how to encourage a bushier structure through strategic pruning and explore methods for propagating new plants from cuttings.
true
1:41:19
Now playing
The ULTIMATE Guide To The 10 Most Popular Plants
258K 2y ago
Sheffield Made Plants
Download my FREE Plant Parent's Troubleshooting Handbook https://resources.sheffieldmadeplants.com/handbook ▸ Book a ...
4K
true
21:20
Now playing
Houseplants I would NEVER buy again! (and some that I would)
54K 1y ago
Kaylee Ellen
Some houseplants just aren't worth it… but a few totally are! Here are the ones I'd buy again. Anthurium Rare Plant Index: ...
Summary
Kaylee Ellen shares her personal experiences with specific plant varieties, explaining why some species prove to be too high-maintenance for a relaxing collection. The discussion covers growth patterns, light requirements, and the effort needed to keep these popular indoor plants looking their best over time.
true
16:09
Now playing
Making Thick Trunk Bonsai from Jade Plant | in 2 Years | Pruning | Repotting | Portulacaria Afra
3.6M 3y ago
What a Plant
Now You can Buy My Premium Bonsai Soil Mix (From Link Below) https://amzn.to/358xd84 Appreciate Your Support Please ...
13 chapters
Intro | Pruning | Preparing Bonsai Pot | Repotting | 2nd Pruning | 1st Result | Repotting into Bigger Pot | 3rd Pruning | Preparing Bonsai Pot | Repotting | Growth After Few Months | 4th Pruning | Final Result
true
17:04
Now playing
AFTER I CREATED THIS METHOD I NEVER LOST AN ORCHIDS AGAIN
2.1M 1y ago
Wonder Garden
Welcome to Wonder Garden! In today's video, discover the ultimate game-changing method that ensures you'll never lose an ...
3 products
4K
Shorts
😲 8FT TALL Monstera Deliciosa Repot 🌿 PART 1 #monstera #houseplants #indoorplants #repotting #plants
13K views
Monstera Deliciosa Aurea Variegated 🪴✨🌙 #plants #plantlover #indoorplants #monstera #houseplants
7.3K views
New
She’s thriving ✨😍 #monstera #monsteradeliciosa #giantmonstera #plants #houseplants
961 views
New
5 Poisonous Fruits You Should Never Eat Unripe! ⚠️Monstera Deliciosa.
7.7K views
New
1 year Monstera Bulbasaur TC update. #monstera #philodendron #plantcare #anthurium #plantcaretips
19K views
true
18:21
Now playing
18 Gorgeous Houseplants That Thrive Without Sunlight
839K 1y ago
Natural Health Remedies
18 Gorgeous Houseplants That Thrive Without Sunlight Don't have a sunny window? No problem! In this video, we're showing ...
Summary
Natural Health Remedies explores diverse options for dimly lit spaces, from vining plants to flowering varieties. Learn specific care tips for each plant, including watering frequency and soil preferences, to ensure longevity without direct sunlight.
true
25:13
Now playing
Just one slice of onion can make Anthurium explode with so many beautiful flowers
542K 1y ago
Wonder Garden
Welcome to Wonder Garden! In today's video, discover an astonishing trick: just one slice of onion can make your Anthurium ...
2 products
4K
true
3:38
Now playing
STOP doing this to your Monstera Deliciosa
349K 4y ago
Kill This Plant
I get the idea behind it. But this is not doing what you think its doing. We need to stop pruning our Monstera Deliciosa. Pruning this ...
4K
Summary
Kill This Plant explains how pruning affects hormone pathways and growth in Monsteras, unlike branching plants like jade. They reveal how cutting stems reverts plants to a juvenile state with smaller leaves and fenestrations.
true
14:42
Now playing
The only 4 Monstera Deliciosa tips you need to know
91K 5mo ago
Kill This Plant
Download My Houseplant App - https://blackthumbplant.app/ Join Our Discord for plant help - https://discord.gg/kw9FnABCRk ...
4K
5 chapters
Find the back of the plant | Measure your light | Get grow lights | Take top cuttings | Conclusion
true
11:01
Now playing
Tips + Tricks For Monstera Deliciosa | ALL YOU NEED TO KNOW 🌿 Indoor Plant Tips For Beginners
47K 5y ago
The Jungle Haven
Monstera deliciosa care tips - swiss cheese plant care - monstera houseplant tips and tricks - indoor plant tips for beginners ...
true
41:52
Now playing
How to continuously grow a large Monstera
146K 1y ago
Sydney Plant Guy
I am chopping my Monstera adansonii in half & re-extend both: one with Moss to grow indoors, one with aroid mix to grow ...
4K
Summary
Sydney Plant Guy demonstrates the 'chop and extend' process to manage a large Monstera adansonii in an indoor setting. The video covers techniques for maintaining plant stability using moss poles and garden stakes while preparing for future growth.
Shorts
5 years of monstera growth! #monstera #plants
696K views
Repotting my NEW indoor monstera plant🪴 #plants #monstera #indoorplants #houseplants #monsteraplant
315K views
New
Converting my M. deliciosa ‘brazil form’ from grown on wood to grown on a moss pole
26K views
Deliciosa VS Borsigiana Monstera | They’re VERY Different - Which One is Yours?
203K views
I have 600 houseplants and my Monstera Deliciosa my absolute FAVORITE plant in my collection 🤍🪴🥹
83K views
true
5:46
Now playing
How to Grow Monstera Deliciosa
6.5K 3mo ago
Tanner The Planter
Day 21/30 of teaching you how to grow your houseplants in a simple way. My reference guide to growing 150 of the most popular ...
Summary
Tanner The Planter provides a comprehensive guide on Monstera deliciosa care, focusing on proper soil composition, effective watering techniques, and optimal lighting conditions. Learn how to encourage healthy root systems and leaf maturity while managing growth through staking.
true
10:15
Now playing
cutting down my GIANT Monstera Deliciosa / Taking Cuttings / Repotting / Feed & Care Nicole's Garden
80K 2y ago
Nicole’s Garden
Please Like and Subscribe! Affiliate links Ponytail sunhat (CAN) https://amzn.to/3UKGeeG Hori Hori Knife (CAN) ...
true
17:59
Now playing
Proper care for Monstera deliciosa! These 7 care tips are a MUST-know 🪴 (Swiss cheese plant)
40K 9mo ago
crazyplantboy
Want your Monstera deliciosa to grow better and healthier? Need an overview of the most important care tips so you can ...
4K
true
2:42
Now playing
239 Days Monstera Deliciosa Time-Lapse – From Seed to Stunning Plant! 🌱
83K 1y ago
Interesting as FCK
Ever wondered how long it really takes to grow a Monstera deliciosa from seed? Here's the full 239-day journey captured in one ...
4K
true
2:52
Now playing
How to Repot Monstera Deliciosa. IT'S EASY!
149K 3y ago
Tanner The Planter
In this video we discuss and demonstrate the general methods of properly re-potting the Monstera Deliciosa. Find Your PERFECT ...
5 chapters
Intro | Preparing the soil | Checking the roots | Repot | Watering
true
19:41
Now playing
Monstera Deliciosa: From Jungle Survivor to Icon 🌿
19K 1y ago
Plantrums
Watch next: The Orchid's Dark Past — Obsession, Mystery, and Survival → https://www.youtube.com/watch?v=q7VpOcKEpAw ...
4 products
7 chapters
Intro | Monstera’s Ancient History | Monstera’s Early Documentation | Monstera’s Biological Secrets | Monstera’s Rise to Fame | Monstera’s Ecological Impact | Monstera’s True Value
Shorts
Eating Monstera Deliciosa Fruit, Finally!
471K views
What to do with your monstera aerial roots? #monsteraplant #plantcaretips #plantcare
981K views
New
Monstera deliciosa (exotic plant that produces fruit) #shorts  #foodforest #tropical  #gardening
1.1K views
A lot of growth already in a year and a half for big mama Monstera Deliciosa. #monsteraplant
433K views
Monstera Deliciosa Repot #monstera #monsteraplant #repotting
49K views
true
11:25
Now playing
How to start Monstera the right way
61K 4mo ago
Kill This Plant
Download My Houseplant App - https://blackthumbplant.app/ Join Our Discord for plant help - https://discord.gg/kw9FnABCRk ...
4K
Summary
Kill This Plant provides a comprehensive guide to maximizing growth for single-stem Monsteras. The discussion covers essential care routines, including light intensity monitoring, selecting appropriate soil amendments to improve drainage, proper staking techniques, and safe fertilizing schedules to help plants thrive.
true
5:37
Now playing
How to Add Support & Prune Back Overgrown Monstera Deliciosa Plants
242K 3y ago
My City Plants
How to prune Monstera Deliciosa (complete guide) ...
5 products
Earns commission
Summary
My City Plants demonstrates techniques for managing large foliage using bamboo stakes, raffia, and clean pruners. Learn how to secure sprawling stems for a narrower profile and selectively cut back growth to maintain a balanced shape.
true
7:13
Now playing
You don’t need to repot your Monstera
80K 2y ago
Kill This Plant
Join our discord community https://discord.gg/RYXzghkuhk * Get 15% off Soltech Grow Lights with promo code: KTP2025 ...
4K
Summary
Kill This Plant explains how to determine when a Monstera actually needs a larger container based on root volume and soil moisture. Learn to identify the right balance of root mass to soil, and discover alternative solutions for managing water retention and light requirements.
true
19:37
Now playing
Saving a Giant: I got an 11+ year old Monstera Deliciosa Large Form
210K 3y ago
Leafy_Luster
MONSTERA DELICIOSA large form exists & I finally found one to call my own. Join me on this Massive Monstera Makeover to ...
4K
true
12:31
Now playing
The Full Story of my 11+ year old Monstera Deliciosa Large Form
108K 2y ago
Leafy_Luster
It's one of my most requested videos to make. THE UPDATE on my rescue Monstera Deliciosa! My rescue mission was quite ...
4K
Shorts
Repotting 7ft+ monstera #shorts #gardenup #monstera #plants
409K views
Monstera Fruit: The Strange Tropical Fruit 🍃🟢
102K views
let's propagate monstera #monstera #propagate #plants #soil #water #roots #areal #cutting #plants
1M views
Grow a BIG Monstera✨ #houseplants #plantcare #plantcaretips #monsteradeliciosa
3.7K views
If you love your monstera, set it free #houseplant #monstera #plantgrowth
351K views
true
14:33
Now playing
MONSTERA PROPAGATION: How to Propagate Monstera deliciosa (2023)
153K 4y ago
Plant Vibrations with Devin Wallien
My favorite Houseplant tools here: https://amzn.to/3sYJ3ta Yesssss, finally I have created a video on how to successfully ...
CC
true
5:00
Now playing
5 quick tips Monstera Deliciosa
88K 5y ago
Swedish Plantguys
In this video we give you 5 quick tips for your Monstera Deliciosa also know as Swiss cheese plant. Enjoy! And thank you for ...
4K
7 chapters
Intro | Juvenile Monstera | Repot | Support | Prune | Humidity | Outro
true
14:29
Now playing
How to Repot a Monstera Deliciosa Plant | Staking | Aerial Roots | Care Guide
37K 3y ago
Ana Perez
Hello friends, here's how to repot a monstera deliciosa plant and how to care for a monstera plant as well! Here are all the ...
4K
CC
12 chapters
intro | When to repot a monstera deliciosa? | What to do when a monstera is really root bound? | What type of potting soil to use for a monstera deliciosa? | What type of pot to use for a monstera? | Are moss poles necessary for a monstera? | What to do with the aerial roots of a monstera | What type of light does a monstera need? | How often should you water a monstera? | How much water should you give a monstera plant? | Fertilizing a monstera plant | Pest Preventions
true
17:27
Now playing
Monstera Deliciosa Single Node Propagation (0-9 Months)
609K 4y ago
Techplant
propagating monstera nodes. In this video I will show you the process of making single node cuttings of monstera deliciosa and ...
1 product
4K
true
9:13
Now playing
5 Things Monsteras HATE That Might Surprise You
387K 3y ago
Sheffield Made Plants
Monsteras aren't particularly fussy plants but there are some key things they hate that might actually surprise you.
Summary
Sheffield Made Plants explains how common care habits like misting, incorrect potting, and allowing dust buildup can harm Monsteras. Learn about the importance of stable humidity, proper root support, and leaf hygiene for optimal plant health and leaf development.
true
11:47
Now playing
MONSTERA DELICIOSA Care | Swiss Cheese Plant Tips & Tricks
187K 7y ago
Good Growing
For bonus content, subscribe to my Patreon ›› https://www.patreon.com/goodgrowing Do you like being trendy? Want plants to ...
11 chapters
Intro | General Info | Water | Light | Temperature | Humidity | Fertiliser | Propagation | Toxicity | Growth Care | Outro
Shorts
Monstera Propagation (My Step by Step Process)
313K views
I CHOPPED UP my Monstera to get it to thrive #houseplants #plants #houseplant #monstera
804K views
Monstera Deliciosa 🌿 Complete Care, Propagation, Height & Fruit Facts!#youtubeshorts#shorts#reels
2.2K views
New
Propagation Loop♻️🌿 #planttips #plants #monstera #propagation #plantcare #diy
18K views
Monstera Deliciosa 2023 Season is Here
1.8M views
true
10:39
Now playing
MONSTERA DELICIOSA CARE + PROPAGATION | indoor swiss cheese plant care
174K 6y ago
Good & Planty
this MONSTER of a care guide is here monstera leaves going yellow? not getting those fenestrations? is your swiss cheese ...
CC
6 chapters
Intro | Water | Light | Repotting | Temperature Humidity | Propagation
true
10:25
Now playing
Monstera richtig pflegen – der komplette Guide | feey 🌱
129K 3y ago
feey
In diesem Video zeigen wir dir alles, was du zur richtigen Pflege deiner Monstera wissen musst: Dazu zählt der richtige Standort, ...
2:46 Das hier ist der Klassiker der Monsteras: die Monstera deliciosa. Diese findest du bei uns im Shop in ganz vielen verschiedenen ...
4K
CC
true
7:35
Now playing
Watch this before you separate your Monstera
38K 1y ago
Kill This Plant
Get 15% off Soltech Grow Lights with promo code: KTP2025 https://glnk.io/0lyv/13 * GET 10% off the best professional ...
4K
Summary
Kill This Plant explains how to properly assess indoor lighting and choose the correct pot size to prevent root rot after propagating. Using a light meter and moisture probe, they demonstrate the relationship between soil volume, oxygen levels, and plant stress.
true
35:40
Now playing
My Monstera Jungle Mint Started Reverting...So I Chopped It All Back 👀
16K 6mo ago
plantsbymelissa
My Monstera jungle mint started reverting, so it was time to cut it back and restart. In this video I'm chopping it down, resetting the ...
4K
8 chapters
Overview | Time to chop | Repotting the base | Chunky pon | Watering | Cuttings | Back in my plant room | Outro
true
15:38
Now playing
Every Type of Monstera Deliciosa Explained (And Which One to Buy)
85K 2y ago
Kaylee Ellen
M E R C H : https://kayleeellen.creator-spring.com/? Monstera Vareities: ...
Summary
Kaylee Ellen outlines the differences between small and large form Monstera deliciosa, focusing on internodal spacing, growth patterns, and leaf characteristics. Learn which variety best suits your available space and aesthetic preferences through a detailed comparison of popular green and variegated cultivars.
Shorts
The MOST DANGEROUS Fruit? - The Monstera Deliciosa Fruit!
229K views
MONSTERA Propagation with 100% SUCCESS | Watch Me Turn a sad 😢 Plant into 11 Happy 🤓babies!
607K views
Monstera Deliciosa : The Delicious Monster Houseplant ! 🪴🍍#garden  #farming #organic
3.1M views
Monstera deliciosa fruit #asmr
901K views
Would you do this?! Full Monstera Cut Back!
456K views
true
9:13
Now playing
5 Things Monsteras HATE That Might Surprise You
387K 3y ago
Sheffield Made Plants
Monsteras aren't particularly fussy plants but there are some key things they hate that might actually surprise you.
Summary
Sheffield Made Plants explains how common care habits like misting, incorrect potting, and allowing dust buildup can harm Monsteras. Learn about the importance of stable humidity, proper root support, and leaf hygiene for optimal plant health and leaf development.
true
11:00
Now playing
How to water propagate a Monstera Deliciosa
64K 1y ago
Kill This Plant
Download My Houseplant App - https://blackthumbplant.app/ Join Our Discord for plant help - https://discord.gg/kw9FnABCRk ...
4K
Summary
Kill This Plant demonstrates how to take top and mid cuttings for propagation while ensuring healthy root development and foliage maintenance. Learn techniques to prevent rot, choose appropriate pot sizes based on root mass, and mix suitable soil for growth.
Shorts
Monstera deliciosa harvest!
4.4M views
Here’s 4 tips to grow a HUGE Monstera Deliciosa! 🌱🪴
286K views
The Secret, Edible Fruit Hiding on Your Houseplant #monstera #animation
713K views
Monstera deliciosa propagation in water❤️#shorts #youtubeshorts #like #viral #fyp
425K views
Monstera Deliciosa Fruit #shorts
51K views
true
2:41
Now playing
My Mutant Monstera Deliciosa Update
7.9K 2y ago
Techplant
Amazon Affiliate link to light https://amzn.to/3S7wJVN These are amazon affiliate links, they don't change the price, however after ...
4K
true
0:47
Now playing
Monstera Deliciosa Time Lapse (HUGE LEAF)
28K 3y ago
Interesting as FCK
In this plant time lapse, we'll be taking a look at a GIANT monstera deliciosa growing, one of my favorite plants in the world!
4K
true
8:16
Now playing
Monstera Deliciosa LARGE FORM: Repotting & Root Dividing
136K 6y ago
reallybigplant
How to repot an extra large Monstera Deliciosa with a huge developed root system! Join me while I try to root divide this Monstera ...
true
8:16
Now playing
How to Grow and Care for Monstera (Swiss Cheese) Indoors
615K 5y ago
Garden Up
LINKS| Good quality soil : https://gardenup.in/products/ready-potting-soil-mix Succulent soil ...
true
7:41
Now playing
Monstera Roots - 4 Things to do with Aerial Roots - Monstera deliciosa (2023)
189K 3y ago
Plant Vibrations with Devin Wallien
Shop plants here! https://plantvibrations-shop.myshopify.com/ While we're growing our Monstera deliciosa or many other aroid ...
CC
Summary
Devin Wallien explains how aerial roots function as nutrient and water absorbers for the plant. The video demonstrates practical techniques for managing these roots, including guiding them into a moss pole, burying them in the soil, or placing them in water to enhance growth.
true
12:09
Now playing
The Easiest Monsteras to Care For 🌿 Great for Beginners Too!
3K 3mo ago
planty skies
🌿 Monsteras are among the most popular houseplants – but which varieties are truly easy to care for and suitable for beginners ...
4K
true
6:11
Now playing
The Giant Monstera Deliciosa Node Update
12K 1y ago
Techplant
Here are the giant monstera nodes I've had propagating for 2 yeara These are amazon affiliate links, they don't change the price, ...
4K
Summary
Techplant documents a two-year propagation experiment using various methods to revive severely rotted monstera cuttings. Despite initial setbaks and neglect, the video tracks the long-term survival and growth of the nodes under different environmental conditions.
true
2:44
Now playing
How to safely eat the poisonous tropical fruit Monstera Deliciosa
130K 9y ago
SleepyLizard
One of our tastiest tropical fruits is also our most dangerous. The Monstera Deliciousa, known as the "fruit salad fruit" combines ...
Shorts
Look at all that Monstera Deliciosa #fruit  #art #india #photogr #reels #bhfyp #explorepage #garden
1.2M views
"Root rot almost killed Monstera… but I brought it back" #plants #gardening
481K views
Monstera Deliciosa Fruit!!!
50K views
Tips to keep your monstera deliciosa alive #plantcare #monsteraplant #monstera
1.1M views
😲 8FT TALL Monstera Deliciosa Repot 🌿 PART 1 #monstera #houseplants #indoorplants #repotting #plants
13K views
true
8:57
Now playing
Monstera Deliciosa Review - Weird Fruit Explorer - Ep. 108
303K 11y ago
Weird Explorer
Help support my series by checking out: My Patreon Page: https://www.patreon.com/weirdexplorer ---- Or get a TSHIRT: ...
CC
true
3:25
Now playing
Monstera deliciosa learns its care easily
122K 3y ago
El De Las Plantas
In this video, I'll show you a quick and easy way to keep your Monstera Deliciosa healthy and happy. #plants ...
4K
true
4:11
Now playing
How to Set Your Monstera For Success // Monstera Deliciosa Plant Care
38K 3y ago
Plantalogy
2 mistakes can hinder the growth of your monstera. These are some common mistakes plant growers often do but rarely notice ...
Summary
Plantalogy highlights two common mistakes regarding pot size and planting depth that lead to root rot and yellowing leaves. Learn proper repotting techniques and how to provide adequate vertical support for a healthy, lush plant.
true
5:47
Now playing
Monstera Plant Care for Beginners (Do This First!)
10K 1y ago
The Girl with a Shovel
Keywords: monstera houseplant, monstera deliciosa care, monstera plant care indoors, how to support monstera plant, saving a ...
CC
7 chapters
Intro | Light Requirements | Watering Requirements | Soil Requirements | Fertilizer Requirements | Humidity Requirements | Bonus Tips
true
20:37
Now playing
The Tastiest Fruit In The World! - How to Pick and Eat Monstera Deliciosa Safely
18K 1y ago
Fruitful Trees
Tom at www.Guacfarm.com shows us how to pick and eat the Monstera Deliciosa Fruit safely. He also teaches me how to correctly ...
true
30:30
Now playing
Monstera Deliciosa Types! | Variegated and Non-Variegated
61K 4y ago
Kaylee Ellen
Head to https://squarespace.com/kayleeellen to save 10% off your first purchase of a website or domain using code ...
Shorts
monstera leaves not splitting? do this! 🌱
382K views
I did the big monstera deliciosa chop… now it’s time to propagate! #monsteraplant #propagation
391K views
Look at how BIG this pretty Monstera is! #plants #monsteradeliciosa #plantmom #propagation
110K views
How To Propagate A Monstera Deliciosa #propagation #monsteradeliciosa #plantcaretips #plantcare
468K views
Repotting my very rootbound Monstera Thai constellation #plantcare #monsterathaiconstellation
661K views
true
5:21
Now playing
HOUSEPLANT CARE TIPS | HOW TO CARE FOR MONSTERA ADANSONII SWISS CHEESE
77K 4y ago
Succulents Box
In this video, you'll find all the care tips for Monstera Swiss Cheese. Time code 0:00 Intro 0:39 Lighting 0:59 Watering 1:41 ...
10 chapters
Intro | Lighting | Watering | Temperature | Soil | Pruning | Propagation | Troubleshooting | Toxicity | Outro
true
4:01
Now playing
Monstera Rehab. How to properly prune your Monstera deliciosa 🪴
235K 4y ago
Paul’s & Sandy’s Too 🪴Karens Content 🪴
Rehab pruning a damaged Monstera! What you need to know.
Summary
After a shipment arrived with heat-damaged plants, Karen demonstrates how to rescue a Monstera deliciosa. The tutorial focuses on identifying the proper spot to prune without removing future leaf growth, a technique applicable to other plants with similar growth patterns.
true
11:32
Now playing
I Stopped Killing Monsteras Once I Knew This
25K 10mo ago
Sheffield Made Plants
Head to https://squarespace.com/sheffield to save 10% off your first purchase of a website or domain using code SHEFFIELD ...
4K
Summary
Learn how to optimize lighting and watering for faster leaf growth and more holes in Monstera plants. Sheffield Made Plants covers essential potting media, managing aerial roots, and troubleshooting pruning techniques.
true
13:59
Now playing
Monstera Deliciosa Care: 7 Things That Keep It Happy & Growing
251 3w ago
Her green space
Monstera deliciosa is one of the easiest houseplants to grow once you understand what it really needs — but small leaves, ...
30 products
15 chapters
Monstera Deliciosa Care | How much light does Monstera need? | Why your Monstera has small leaves or no fenestrations | How I water my Monstera | Best soil mix for Monstera | When should you repot? | Does Monstera really need a moss pole? | What to do with Monstera aerial roots | How much humidity does Monstera need? | Fertilizing Monstera | My Monstera growing in LECA | Yellow leaves, brown leaves & common problems | Cleaning leaves & checking for pests | Pests | What keeps A monstera happy
true
12:10
Now playing
MONSTERA DELICIOSA CARE TIPS
152K 6y ago
Green Yard TV
Monstera Deliciosa Care tips I hope you find this video helpful and interesting! If you did, please don't forget to hit LIKE, ...
true
10:39
Now playing
How to properly care for Monstera Thai Constellation – Everything you need to know! 🌿✨
20K 1y ago
planty skies
🌿✨ The Monstera deliciosa Thai Constellation is one of the most beautiful and sought-after houseplants – but how do you care ...
4K
true
9:47
Now playing
Everything You Need to Know About MONSTERA DELICIOSA | Monstera deliciosa plant care | Bagicha Vl...
34K 4y ago
Drakht
The Video Content is not intended to be a substitute for professional medical advice, diagnosis, or treatment. Always seek the ...
true
16:24
Now playing
Everything about MONSTERA DELICIOSA | Growth and care guide
175K 4y ago
I Giardini di Ellis
► Read my article on Monstera: https://igiardinidiellis.it/monstera-deliciosa/ ► Support your plants https://www.amazon.it ...
CC
9 chapters
Intro | PROVENIENZA e nome della Monstera deliciosa | LUCE e temperatura ideale per la Monstera deliciosa | ACQUA e umidità ideale per la Monstera deliciosa | RINVASI e terriccio per la Monstera deliciosa | PARTICOLARITÀ della Monstera deliciosa | CURA per la Monstera deliciosa | PROPAGAZIONE per la Monstera deliciosa | MEGLIO SAPERLO
true
11:46
Now playing
The Alternative Way To Grow The Best Monstera
79K 2y ago
Sheffield Made Plants
Head to https://squarespace.com/sheffield to save 10% off your first purchase of a website or domain using code SHEFFIELD ...
4K
8 chapters
Intro | Root Rot | Top cuttings | Vase | Stones | Assembling | Watering | pH
true
12:21
Now playing
The truth about monstera care!!!!
35K 1y ago
addictoplantes
Everything you need to know about caring for Monsteras! Watering, lighting, repotting, pests... I'll tell you everything you ...
true
16:37
Now playing
Meet All Variegated Monstera Varieties And Expert Care Tips @noldy.topan
115K 3y ago
OnlyPlants
In this tour, we cover the most extensive variegated monstera collector in Indonesia - Mr Noldy Topan of @dutahijauabadi.
6:55 ... the monstera deliciosa called the Sphinx Mr Noli got this from Thailand and it's well known for its sectoral variegation this means ...
4K
true
26:17
Now playing
Airlayering, chopping, and repotting my EXTRA large Monstera Deliciosa!
80K 2y ago
PlantswithKrystal
Hey hey! Today's video shows the process of air layering, chopping, and repotting my large monstera. Hope it gives you a little ...
1 product
Summary
After successfully air layering a massive Monstera Deliciosa, Krystal proceeds with a daring chop to manage its height. The process involves transferring the propagated cutting into a new, larger pot alongside the original plant to create a fuller appearance.
true
1:12
Now playing
Comment entretenir ma monstera ? - Truffaut
70K 7y ago
TRUFFAUT
Découvrez le Monstera et ses secrets d'entretien ! Plante iconique des intérieurs végétalisés, le Monstera Deliciosa, aussi ...
true
18:15
Now playing
Monstera deliciosa: Easy propagation and tips for making it grow beautifully.
178K 3y ago
Ivet Suculentas y mas
Hello plant lovers! This is your friend Ivet, from Oaxaca, who is completely in love with plants. My little paradise is ...
4K
Summary
Ivet muestra cómo propagar una monstera deliciosa para recuperar su color blanco característico, utilizando un sustrato ligero a base de turba de coco y perlita. La demostración incluye el corte estratégico del tallo y el uso de tutores para asegurar raíces fuertes y frondosidad.
true
20:45
Now playing
Why Is My Variegated Monstera Reverting? (Causes & How to Fix It!)
2.4K 8mo ago
Jamie Rootledge
Is your variegated Monstera reverting back to green? In this video, I break down why variegated Monsteras (Albo, Thai ...
Summary
Jamie Rootledge demonstrates how to propagate a reverted jungle mint Monstera to encourage the return of variegated growth. The process involves identifying healthy nodes on the stem, making strategic cuttings, and establishing them in moss to foster new, colorful foliage.
true
4:30
Now playing
Costilla de Adán o Monstera Deliciosa: características y reproducción - Decogarden - Jardinatis
84K 6y ago
Jardinatis
Iñigo Segurola nos muestra las enormes variedades de Costilla de Adán que tiene en su invernadero y nos enseña las ...
25 videos
Monstera Essentials
Kill This Plant
Playlist
How to set up your Monstera for success · 3:58
why people keep Killing their Monstera · 3:13
View full playlist
If your Monstera Deliciosa is struggling, check out these essential videos to help troubleshoot your problems.
true
13:15
Now playing
Your Monstera Will NEVER Thrive Unless You Do This
141K 1y ago
Sheffield Made Plants
Head to https://squarespace.com/sheffield to save 10% off your first purchase of a website or domain using code SHEFFIELD ...
4K
true
42:26
Now playing
My Variegated Monstera Deliciosa Collection 🌿
30K 1y ago
plantsbymelissa
In this video, I'm sharing a closer look at all the stunning variegated monstera deliciosas in my collection, including albo, aurea, ...
4K
9 chapters
intro | Monstera Deliciosa ‘Mint’ (NOID, non-TC) | Monstera Deliciosa ‘Jungle Mint’ (NOID, non-TC) | Monstera Deliciosa ‘Albo Variegata’ (White Tiger type) | Monstera Deliciosa ‘Mint’ (tissue culture) | Monstera Deliciosa ‘Aurea’ | Monstera Deliciosa ‘Thai Constellation’ | Monstera Deliciosa ‘Albo Variegata’ | outro
true
17:59
Now playing
Monstera Mania! | All of the Monstera Deliciosa I am Growing!
3.1K 6mo ago
PlantswithJacque
Monstera Deliciosa Types Showcase In this video, I'm walking you through my collection of Monstera deliciosa varieties and ...
Summary
PlantswithJacque showcases a diverse collection of Monstera deliciosa varieties, detailing unique variegation patterns and specific growth forms. Each plant is presented with insights into its care requirements, preferred growing conditions, and individual developmental stages to help viewers distinguish between these popular botanical forms.
true
10:03
Now playing
The Ultimate Guide to Monstera Care | How to Grow Big, Healthy Monstera Plants Indoors
7.8K 1y ago
The Cozy Jungle
Whether you're a beginner or an experienced plant parent, this video will help you keep your Monstera deliciosa looking vibrant ...
4K
true
4:56
Now playing
Monstera deliciosa (Swiss Cheese Plant) Care 🌿 Watering, Propagation, Pruning and more!
118K 5y ago
EcologíaVerde
🌿 Are you thinking about growing a Monstera deliciosa (Swiss cheese plant) indoors? In this Green Ecology video, we bring you ...
7 chapters
Cuidados de la costilla de Adán o Monstera deliciosa | Temperatura adecuada | Riego de la Monstera deliciosa | Luz y ubicación ideales | Poda recomendada | Hojas amarillas en la Monstera deliciosa | Cómo reproducir la planta costilla de Adán
true
1:09:24
Now playing
Growing & chopping up my huge Monstera adansonii (includes 12 months of updates)
31K 1y ago
Sydney Plant Guy
Let's grow my Monstera adansonii even larger over the course of a full year! watch my review of the grow light: ...
4K
Summary
Sydney Plant Guy tracks the recovery of a Monstera adansonii after an aggressive pruning session. The journey includes restructuring the plant's support pole and handling unexpected flowering to manage growth over the course of a full year.
true
30:59
Now playing
$15 BIG Monstera Deliciosa at 50% OFF HOME DEPOT PLANT SALE! Prices, Pet Safe Plants, Rare Plants
15K 2mo ago
Plant Life with Ashley Anita
plants #homedepotfinds #plantshopping *This is a good vibes only channel for all plant friends to enjoy! :) Any negative and ...
trueonditions.
true
2:44
Now playing
How to safely eat the poisonous tropical fruit Monstera Deliciosa
130K 9y ago
SleepyLizard
One of our tastiest tropical fruits is also our most dangerous. The Monstera Deliciousa, known as the "fruit salad fruit" combines ...
Shorts
Look at all that Monstera Deliciosa #fruit  #art #india #photogr #reels #bhfyp #explorepage #garden
1.2M views
"Root rot almost killed Monstera… but I brought it back" #plants #gardening
481K views
Monstera Deliciosa Fruit!!!
50K views
Tips to keep your monstera deliciosa alive #plantcare #monsteraplant #monstera
1.1M views
😲 8FT TALL Monstera Deliciosa Repot 🌿 PART 1 #monstera #houseplants #indoorplants #repotting #plants
13K views
true
8:57
Now playing
Monstera Deliciosa Review - Weird Fruit Explorer - Ep. 108
303K 11y ago
Weird Explorer
Help support my series by checking out: My Patreon Page: https://www.patreon.com/weirdexplorer ---- Or get a TSHIRT: ...
CC
true
3:25
Now playing
Monstera deliciosa learns its care easily
122K 3y ago
El De Las Plantas
In this video, I'll show you a quick and easy way to keep your Monstera Deliciosa healthy and happy. #plants ...
4K
true
4:11
Now playing
How to Set Your Monstera For Success // Monstera Deliciosa Plant Care
38K 3y ago
Plantalogy
2 mistakes can hinder the growth of your monstera. These are some common mistakes plant growers often do but rarely notice ...
Summary
Plantalogy highlights two common mistakes regarding pot size and planting depth that lead to root rot and yellowing leaves. Learn proper repotting techniques and how to provide adequate vertical support for a healthy, lush plant.
true
5:47
Now playing
Monstera Plant Care for Beginners (Do This First!)
10K 1y ago
The Girl with a Shovel
Keywords: monstera houseplant, monstera deliciosa care, monstera plant care indoors, how to support monstera plant, saving a ...
CC
7 chapters
Intro | Light Requirements | Watering Requirements | Soil Requirements | Fertilizer Requirements | Humidity Requirements | Bonus Tips
true
20:37
Now playing
The Tastiest Fruit In The World! - How to Pick and Eat Monstera Deliciosa Safely
18K 1y ago
Fruitful Trees
Tom at www.Guacfarm.com shows us how to pick and eat the Monstera Deliciosa Fruit safely. He also teaches me how to correctly ...
true
30:30
Now playing
Monstera Deliciosa Types! | Variegated and Non-Variegated
61K 4y ago
Kaylee Ellen
Head to https://squarespace.com/kayleeellen to save 10% off your first purchase of a website or domain using code ...
Shorts
monstera leaves not splitting? do this! 🌱
382K views
I did the big monstera deliciosa chop… now it’s time to propagate! #monsteraplant #propagation
391K views
Look at how BIG this pretty Monstera is! #plants #monsteradeliciosa #plantmom #propagation
110K views
How To Propagate A Monstera Deliciosa #propagation #monsteradeliciosa #plantcaretips #plantcare
468K views
Repotting my very rootbound Monstera Thai constellation #plantcare #monsterathaiconstellation
661K views
true
5:21
Now playing
HOUSEPLANT CARE TIPS | HOW TO CARE FOR MONSTERA ADANSONII SWISS CHEESE
77K 4y ago
Succulents Box
In this video, you'll find all the care tips for Monstera Swiss Cheese. Time code 0:00 Intro 0:39 Lighting 0:59 Watering 1:41 ...
10 chapters
Intro | Lighting | Watering | Temperature | Soil | Pruning | Propagation | Troubleshooting | Toxicity | Outro
true
4:01
Now playing
Monstera Rehab. How to properly prune your Monstera deliciosa 🪴
235K 4y ago
Paul’s & Sandy’s Too 🪴Karens Content 🪴
Rehab pruning a damaged Monstera! What you need to know.
Summary
After a shipment arrived with heat-damaged plants, Karen demonstrates how to rescue a Monstera deliciosa. The tutorial focuses on identifying the proper spot to prune without removing future leaf growth, a technique applicable to other plants with similar growth patterns.
true
11:32
Now playing
I Stopped Killing Monsteras Once I Knew This
25K 10mo ago
Sheffield Made Plants
Head to https://squarespace.com/sheffield to save 10% off your first purchase of a website or domain using code SHEFFIELD ...
4K
Summary
Learn how to optimize lighting and watering for faster leaf growth and more holes in Monstera plants. Sheffield Made Plants covers essential potting media, managing aerial roots, and troubleshooting pruning techniques.
true
13:59
Now playing
Monstera Deliciosa Care: 7 Things That Keep It Happy & Growing
251 3w ago
Her green space
Monstera deliciosa is one of the easiest houseplants to grow once you understand what it really needs — but small leaves, ...
30 products
15 chapters
Monstera Deliciosa Care | How much light does Monstera need? | Why your Monstera has small leaves or no fenestrations | How I water my Monstera | Best soil mix for Monstera | When should you repot? | Does Monstera really need a moss pole? | What to do with Monstera aerial roots | How much humidity does Monstera need? | Fertilizing Monstera | My Monstera growing in LECA | Yellow leaves, brown leaves & common problems | Cleaning leaves & checking for pests | Pests | What keeps A monstera happy
true
12:10
Now playing
MONSTERA DELICIOSA CARE TIPS
152K 6y ago
Green Yard TV
Monstera Deliciosa Care tips I hope you find this video helpful and interesting! If you did, please don't forget to hit LIKE, ...
true
10:39
Now playing
How to properly care for Monstera Thai Constellation – Everything you need to know! 🌿✨
20K 1y ago
planty skies
🌿✨ The Monstera deliciosa Thai Constellation is one of the most beautiful and sought-after houseplants – but how do you care ...
4K
true
9:47
Now playing
Everything You Need to Know About MONSTERA DELICIOSA | Monstera deliciosa plant care | Bagicha Vl...
34K 4y ago
Drakht
The Video Content is not intended to be a substitute for professional medical advice, diagnosis, or treatment. Always seek the ...
true
16:24
Now playing
Everything about MONSTERA DELICIOSA | Growth and care guide
175K 4y ago
I Giardini di Ellis
► Read my article on Monstera: https://igiardinidiellis.it/monstera-deliciosa/ ► Support your plants https://www.amazon.it ...
CC
9 chapters
Intro | PROVENIENZA e nome della Monstera deliciosa | LUCE e temperatura ideale per la Monstera deliciosa | ACQUA e umidità ideale per la Monstera deliciosa | RINVASI e terriccio per la Monstera deliciosa | PARTICOLARITÀ della Monstera deliciosa | CURA per la Monstera deliciosa | PROPAGAZIONE per la Monstera deliciosa | MEGLIO SAPERLO
true
11:46
Now playing
The Alternative Way To Grow The Best Monstera
79K 2y ago
Sheffield Made Plants
Head to https://squarespace.com/sheffield to save 10% off your first purchase of a website or domain using code SHEFFIELD ...
4K
8 chapters
Intro | Root Rot | Top cuttings | Vase | Stones | Assembling | Watering | pH
true
12:21
Now playing
The truth about monstera care!!!!
35K 1y ago
addictoplantes
Everything you need to know about caring for Monsteras! Watering, lighting, repotting, pests... I'll tell you everything you ...
true
16:37
Now playing
Meet All Variegated Monstera Varieties And Expert Care Tips @noldy.topan
115K 3y ago
OnlyPlants
In this tour, we cover the most extensive variegated monstera collector in Indonesia - Mr Noldy Topan of @dutahijauabadi.
6:55 ... the monstera deliciosa called the Sphinx Mr Noli got this from Thailand and it's well known for its sectoral variegation this means ...
4K
true
26:17
Now playing
Airlayering, chopping, and repotting my EXTRA large Monstera Deliciosa!
80K 2y ago
PlantswithKrystal
Hey hey! Today's video shows the process of air layering, chopping, and repotting my large monstera. Hope it gives you a little ...
1 product
Summary
After successfully air layering a massive Monstera Deliciosa, Krystal proceeds with a daring chop to manage its height. The process involves transferring the propagated cutting into a new, larger pot alongside the original plant to create a fuller appearance.
true
1:12
Now playing
Comment entretenir ma monstera ? - Truffaut
70K 7y ago
TRUFFAUT
Découvrez le Monstera et ses secrets d'entretien ! Plante iconique des intérieurs végétalisés, le Monstera Deliciosa, aussi ...
true
18:15
Now playing
Monstera deliciosa: Easy propagation and tips for making it grow beautifully.
178K 3y ago
Ivet Suculentas y mas
Hello plant lovers! This is your friend Ivet, from Oaxaca, who is completely in love with plants. My little paradise is ...
4K
Summary
Ivet muestra cómo propagar una monstera deliciosa para recuperar su color blanco característico, utilizando un sustrato ligero a base de turba de coco y perlita. La demostración incluye el corte estratégico del tallo y el uso de tutores para asegurar raíces fuertes y frondosidad.
true
20:45
Now playing
Why Is My Variegated Monstera Reverting? (Causes & How to Fix It!)
2.4K 8mo ago
Jamie Rootledge
Is your variegated Monstera reverting back to green? In this video, I break down why variegated Monsteras (Albo, Thai ...
Summary
Jamie Rootledge demonstrates how to propagate a reverted jungle mint Monstera to encourage the return of variegated growth. The process involves identifying healthy nodes on the stem, making strategic cuttings, and establishing them in moss to foster new, colorful foliage.
true
4:30
Now playing
Costilla de Adán o Monstera Deliciosa: características y reproducción - Decogarden - Jardinatis
84K 6y ago
Jardinatis
Iñigo Segurola nos muestra las enormes variedades de Costilla de Adán que tiene en su invernadero y nos enseña las ...
25 videos
Monstera Essentials
Kill This Plant
Playlist
How to set up your Monstera for success · 3:58
why people keep Killing their Monstera · 3:13
View full playlist
If your Monstera Deliciosa is struggling, check out these essential videos to help troubleshoot your problems.
true
13:15
Now playing
Your Monstera Will NEVER Thrive Unless You Do This
141K 1y ago
Sheffield Made Plants
Head to https://squarespace.com/sheffield to save 10% off your first purchase of a website or domain using code SHEFFIELD ...
4K
true
42:26
Now playing
My Variegated Monstera Deliciosa Collection 🌿
30K 1y ago
plantsbymelissa
In this video, I'm sharing a closer look at all the stunning variegated monstera deliciosas in my collection, including albo, aurea, ...
4K
9 chapters
intro | Monstera Deliciosa ‘Mint’ (NOID, non-TC) | Monstera Deliciosa ‘Jungle Mint’ (NOID, non-TC) | Monstera Deliciosa ‘Albo Variegata’ (White Tiger type) | Monstera Deliciosa ‘Mint’ (tissue culture) | Monstera Deliciosa ‘Aurea’ | Monstera Deliciosa ‘Thai Constellation’ | Monstera Deliciosa ‘Albo Variegata’ | outro
true
17:59
Now playing
Monstera Mania! | All of the Monstera Deliciosa I am Growing!
3.1K 6mo ago
PlantswithJacque
Monstera Deliciosa Types Showcase In this video, I'm walking you through my collection of Monstera deliciosa varieties and ...
Summary
PlantswithJacque showcases a diverse collection of Monstera deliciosa varieties, detailing unique variegation patterns and specific growth forms. Each plant is presented with insights into its care requirements, preferred growing conditions, and individual developmental stages to help viewers distinguish between these popular botanical forms.
true
10:03
Now playing
The Ultimate Guide to Monstera Care | How to Grow Big, Healthy Monstera Plants Indoors
7.8K 1y ago
The Cozy Jungle
Whether you're a beginner or an experienced plant parent, this video will help you keep your Monstera deliciosa looking vibrant ...
4K
true
4:56
Now playing
Monstera deliciosa (Swiss Cheese Plant) Care 🌿 Watering, Propagation, Pruning and more!
118K 5y ago
EcologíaVerde
🌿 Are you thinking about growing a Monstera deliciosa (Swiss cheese plant) indoors? In this Green Ecology video, we bring you ...
7 chapters
Cuidados de la costilla de Adán o Monstera deliciosa | Temperatura adecuada | Riego de la Monstera deliciosa | Luz y ubicación ideales | Poda recomendada | Hojas amarillas en la Monstera deliciosa | Cómo reproducir la planta costilla de Adán
true
1:09:24
Now playing
Growing & chopping up my huge Monstera adansonii (includes 12 months of updates)
31K 1y ago
Sydney Plant Guy
Let's grow my Monstera adansonii even larger over the course of a full year! watch my review of the grow light: ...
4K
Summary
Sydney Plant Guy tracks the recovery of a Monstera adansonii after an aggressive pruning session. The journey includes restructuring the plant's support pole and handling unexpected flowering to manage growth over the course of a full year.
true
30:59
Now playing
$15 BIG Monstera Deliciosa at 50% OFF HOME DEPOT PLANT SALE! Prices, Pet Safe Plants, Rare Plants
15K 2mo ago
Plant Life with Ashley Anita
plants #homedepotfinds #plantshopping *This is a good vibes only channel for all plant friends to enjoy! :) Any negative and ...
true 30°), and the ends of an open path curve in, so uneven spacing neither overshoots nor loops; the rubber band, dragged, inserted and converted points use the same rule; the Pencil's fitting unchanged (#1236, in part); scores unchanged (estimated) |
| 2026-10-14 | minor | Quick Image Trace presets reveal the adjustment panel with Advanced open; Properties reopens it. Sliders keep their preset row until release so the re-trace is committed. UI regressions added; scores and remaining estimates unchanged. |
| 2026-10-14 | minor | The macOS save sheet's Format menu goes back to missing: #1855 withdrew it to restore saving (#866 reopened). Rows added for the X/Y/W/H of direct-selected anchors (#1839, fixes #1824), the toolbar and All Tools drawer (#1751) and the Actions panel selection (#1853, part of #1629); the snapping row notes Snap to Grid over Snap to Pixel (#1840). Scores unchanged (estimated) |
| 2026-10-14 | minor | Existing-guide Escape cancellation: eight app-input regressions, the Direct Selection Escape control and current full CI passed; scores and hours unchanged. |
| 2026-10-15 | minor | Artboard tool: closing a clean active tab restores the displayed board before Properties Delete. Nine targeted checks passed; three original close-route captures reviewed; scores and hours unchanged. |
| 2026-10-14 | minor | Artboard tab changes align the tool with the displayed board; same-tab multiselection and Move/Scale Artwork options preserved. Three tab regressions and historical full CI on `a0671777` passed; scores and hours unchanged. |
| 2026-10-14 | minor | Width-point finite-arithmetic follow-up: atomicity, history and native round-trip regressions and current full CI passed; scores and hours unchanged. |
| 2026-10-14 | minor | Align and Distribute act on the anchor points selected with Direct Selection: the points of one path align to each other, not to the artboard, and Distribute spaces them (#1641, fixes #1636; aligning to a key anchor is still missing); Delete removes every selected anchor of a closed path (it used to remove the wrong ones), removes objects selected whole along with the selected anchors of others, and keeps the remainder selected so a second Delete removes it (#1674); In isolation mode Direct Selection and Group Selection pick only inside the isolated object (#1666); Escape cancels a Direct Selection drag (#1803, #1743); Escape while typing ends the edit, keeps the type selected and switches to the Selection tool (#1802, #1686, #1656); A filled Pencil path keeps a stroke of None (#1804, part of #1679); With the Pen, a press that moves under 3 screen pixels is a click (a corner without handles), and a Precise Cursors command toggles the existing preference and can take a shortcut (#1663); The [ and ] keys reach the Blob Brush, the Eraser and the Symbolism tools in the app (#1734, part of #1661); In the Pathfinder panel a group selected alone counts its nested shapes, so an expanded stroked object keeps the Shape Modes enabled (#1642, #1635); Object › Expand turns radial, grid and mirror repeats into groups of their instances (#1765, #1756); Select › Same matches type on its characters' fill and stroke (#1746, #1737); Fields commit what was typed when you click away or move to another field, as Enter does; Escape still discards it (#1787); Up and Down move through the command search results and Enter runs the highlighted one (#1798, #1757); Slider rails show on Medium Dark (#1808, #1662); On Windows the custom title bar's window stays truly borderless, so the pointer lines up with what is drawn on systems where the hidden caption offset it (#1791, #1758; awaiting the reporter's confirmation); Tool cursors are painted into the window on Wayland (#1652; not run on Linux, #1645 stays open until confirmed); the guide-position row (#1659) is now "not scored": the documentation found doesn't establish it, and it is kept as a convenience pending the owner's decision; scores unchanged |
| 2026-10-13 | minor | Pathfinder's Trim, Merge and Crop remove the objects' strokes, and Merge joins same-fill objects whatever their strokes were, as the public Pathfinder page describes (#1572, fixes #1570); Offset Path with an inset larger than the shapes says the offset leaves nothing, instead of asking to select paths (#1591, fixes #1590); The Symbols panel's New Symbol opens Symbol Options to name the symbol first, and Alt-click skips it (#1566, fixes #1560); Tool flyouts are as wide as their longest translated name and shortcut need, so names no longer run into shortcuts in Spanish and other languages (#1650, fixes #1616); Smart Guides: a point drawn with the Pen or a shape tool, and an anchor dragged with Direct Selection, snaps to where paths and ruler guides cross, labelled "intersect" (#1658, part of #1602; a moved selection's own anchors don't); The Transform panel shows a selected ruler guide's X or Y, and typing a value moves it (#1659, part of #1597). This is not established by the documentation we could find: the public help pages on guides, rulers and the Transform, Control and Properties panels (read again on 2026-10-14) describe selecting and dragging unlocked guides and the panels' X and Y for a selection, not a guide's position in a panel, and the one public statement of it is a user's [community answer](https://community.adobe.com/questions-652/adding-a-new-custom-guide-785586), which is not documentation. It is kept as a convenience pending the owner's decision, with no parity claim and nothing counted towards a score; Select Behind (Ctrl/Cmd-click) tests each object once, so a deep stack of overlapping objects no longer slows it; same objects in the same order (#1571); Commands that take `ids` run on the objects given with nothing selected, over MCP, the control channel and the CLI too (#1577, fixes #1575); scores unchanged (estimated) |
| 2026-10-13 | minor | G1/M11: Artboard panel deletion, protected deletion keys and navigator/tool synchronization before and after deletion; history normalization preserves valid multiselection; history regressions and full workspace CI pass; scores and hours unchanged |
| 2026-10-16 | minor | Document window scroll bars (#2322): thumb drag, track paging and the wheel over a bar; hidden in Full Screen and Presentation Mode; scores unchanged |
| 2026-10-13 | minor | Alt during a drag from either ruler swaps the new guide's orientation ([public shortcut reference](https://helpx.adobe.com/illustrator/using/default-keyboard-shortcuts.html)); tool and egui event tests cover preview, snapping, release coordinates, cancellation and one-step Undo/Redo; scores unchanged (estimated). |
| 2026-10-13 | minor | G1: create width points through Width Point Edit with optional adjoining adjustment; command/pointer/history regressions and full CI passed; dialog layout reviewed; scores and hours unchanged |
| 2026-10-13 | minor | Corner Radius: a link toggle behind the field in the Properties and Transform panels; off, a live rectangle shows a radius field for each corner (Top Left, Top Right, Bottom Right, Bottom Left), each setting its own corner (`singleCornerRadius` preference); split from #1594; score unchanged (estimated) |
| 2026-10-13 | minor | Join merges the selected anchor points that lie on one another, as after Average, into one anchor (#1582, fixes #1533); correction: "Join with corner/smooth choice" was listed as done, but `path.join` and `path.average` are separate commands and Join has no such choice, so Average and Join in one step with that choice is its own row, missing; the Smooth tool smooths only the stretch of path under the drag and a round corner next to a curved side curves smoothly instead of sticking out of the shape (#1547, fixes #1512); Lock Guides also locks the objects made into guides: they can't be selected, moved or deleted (#1543); they are drawn in the guide colour and style of Preferences, and a dragged selection snaps to ruler guides with Smart Guides on (#1583, fixes #1534); Edit › Keyboard Shortcuts lists Edit › Duplicate and the other commands that are in no menu but run without parameters (panel-menu commands such as Break Link to Symbol), so they can be given shortcuts (#1549, #1580, fixes #1539); Make Compound Path on a group (such as outlined type) keeps the paint of its backmost path instead of drawing nothing, and compound paths joined into one keep their holes (#1537); scores unchanged (estimated) |
| 2026-10-13 | minor | Sliders in Save for Web, the raster export options, Preferences and the opacity popup show their rail (#1506, fixes #1483); a template layer stays in Preview, dimmed, in Outline view (#1505, fixes #1486); Direct Selection takes the selected object's anchor where anchors of overlapping objects coincide (#1509, fixes #1508); a tab stop in the Tabs panel drags from where the press landed (#1487, fixes #1458); scores unchanged (estimated) |
| 2026-10-12 | minor | Width Point Edit: Shift-double-click enables adjoining adjustment; plain double-click keeps it off; scores and hours unchanged |
| 2026-10-13 | minor | Split Into Grid has its row and column sizes, gutters, totals, Add Guides and Preview (#1488, fixes #1462); Transform Objects in the Transform dialogs and Transform Each, and Reflect across an angle (#1494, fixes #1448 and #1460); the Pathfinder panel works on a group selected alone and its greyed-out buttons say what they need (#1493, for #1450); selected text in value fields is white or black on the highlight (#1490, part of #1465); a focused field follows its value when a slider or preview changes it (#1497, fixes #1457); an Alt-drag copies once per drag (#1464); scores unchanged (estimated) |
| 2026-10-12 | minor | Transform dialogs (Move, Rotate, Scale, Reflect, Shear) preview on the canvas and step their angles and percentages with the arrow keys (#1428, fixes #1412); the transform tools, dialogs and Move act on anchors selected with Direct Selection (#1427, fixes #1408); the bounding box hides during a corner drag with the Selection tool (#1402, fixes #1229); a picture pastes when blank text sits beside it on the clipboard (#1404); scores unchanged (estimated) |
| 2026-10-12 | minor | Dialogs opened from a menu take the keyboard focus every time they open (#1367); a first field that starts greyed out takes it once enabled (#1395); art flush with an artboard within a rounding error moves, copies and scales with it (#1375, from #1295); scores and hours unchanged |
| 2026-10-12 | minor | Rectangular Grid and Polar Grid tool options (skew, frame, compound path, Fill Grid; #1377, in part): a skew slider under each divider count, kept as tool options and used by drags; the skew curve is our own; score unchanged (estimated) |
| 2026-10-12 | minor | Live Corners: a Direct Selection click on a widget selects its corner, Shift-click toggles it (#1239); #908 and #1232 stay open; score unchanged (estimated) |
| 2026-10-12 | minor | Custom ruler origin (#1212): corner drag previews and sets it; double-click resets to the active artboard; X/Y fields use it and native files keep it; scores unchanged (estimated) |
| 2026-10-12 | minor | Artboard tool clicks fit the topmost editable object in one undo step (#1227); scores unchanged |
| 2026-10-12 | minor | Pathfinder single-result booleans preserve source names and SVG IDs (#1293); paint-order, grouped/compound sources, unchanged geometry/appearance and Undo/Redo regression tests; scores and estimates unchanged |
| 2026-10-12 | minor | DXF reads control-point SPLINE closure (#1299 follow-up after #1320); open curves stay open and curves retained; rational/non-rational import and export versions regression-tested; scores/hours unchanged |
| 2026-10-12 | minor | Document Setup: Edit Artboards closes the modal after applying valid settings; pointer-click regressions; scores and hours unchanged |
| 2026-10-12 | minor | Artboards: Escape exits editing; an active drag cancels first; shortcut-routing regression; scores and hours unchanged |
| 2026-10-12 | minor | Simplify previews and shows original/new point counts; refits/subdivisions select changed paths whole (#1272); auto/slider/corner-angle UI and Show Original remain partial |
| 2026-10-12 | minor | Texture-effect drag redraws optimized (#1257); idle Windows interaction still unverified; scores unchanged |
| 2026-10-12 | minor | Curvature: circular point markers, no direction handles or duplicate points; existing-path interpolation and #975 remain separate (#1236, in part); scores unchanged (estimated) |
| 2026-10-12 | minor | Direct Selection: repeating anchor Align and rounding-only Align leave Undo, Redo and saved state intact (#1180 follow-up); scores unchanged (estimated) |
| 2026-10-12 | minor | Eyedropper: original pipette at the sampling tip; precise setting and DPI checked (#1269); OS visual QA and Caps Lock unverified; scores unchanged |
| 2026-10-12 | minor | Eyedropper can read visible locked and outside-isolation sources (#1251); scores unchanged |
| 2026-10-12 | minor | Direct Selection: segment click/display and Delete preserve neighboring edges (#1278); pointer/egui/native checks, OS visual QA pending; score unchanged |
| 2026-10-12 | minor | Adding paint to an unpainted zero-width stroke restores the 1 pt default (#1253); scores unchanged |
| 2026-10-12 | minor | macOS native save sheets gain a localised Format menu (#866); filename regression tests and an actual SVG save, PDF selection and cancellation verified; scores unchanged (estimated) |
| 2026-10-12 | minor | Attributes: directly selected compound members read and change the compound fill rule; path direction stays member-local (#1197); score unchanged |
| 2026-10-12 | minor | Pathfinder Divide: open cutting lines split filled paths with one Undo (#1307); scores and hours unchanged |
| 2026-10-12 | minor | Numeric entry: click-to-focus selection waits for release so text drags keep their start (#1326, in part); keyboard focus, double-click, arrows and wheel preserved; scores and hours unchanged; short movements below the click threshold remain open |
| 2026-10-12 | minor | Double Click To Isolate: the Selection tool also isolates a single path, compound path, image, mesh or placed file; only it is hit, marqueed and selected inside; scores and hours unchanged |
| 2026-10-11 | minor | A marquee inside a filled shape no longer selects it: only touching its path does (#1103); score unchanged (estimated) |
| 2026-10-11 | minor | Live Corners: corners between curved sides round too, the curve trimmed to what the cut leaves (#1144); score unchanged (estimated) |
| 2026-10-11 | minor | Artboards: Shift-click selects several, which move, copy and delete together (#1139); score unchanged (estimated) |
| 2026-10-11 | minor | Edit Toolbar opens the All Tools drawer beside the toolbar instead of a centred dialog, dims the tools the toolbar holds, and edits the toolbar by drag and drop while it is open (#1018); score unchanged (estimated) |
| 2026-10-11 | minor | Direct Selection: dragging a path selected whole by an anchor moves it whole; Alt-drag copies by an anchor too, toggles during the drag and snaps onto the original (#1104); score unchanged (estimated) |
| 2026-10-10 | major | First checklist, built from the tool sources and tests against Illustrator 2026's public documentation |

> **Tab-drag status (2026-10-22):** updated from upstream, including the Turkish catalog and all three tab-drag translations requested in #3479; Selection-tool reset regression-checked and clipboard evidence scoped to internal state; drag lifecycle, insertion and regression contracts documented; behavior, scores and estimates unchanged.

| Date | Change | Tab-drag history |
|---|---|---|
| 2026-10-22 | minor | Verify Selection-tool reset after tab activation, drop and cancellation; narrow clipboard evidence to the tested internal clipboard and correct documentation grammar; behavior, scores and estimates unchanged. |
| 2026-10-22 | minor | Merge current main and complete the Turkish tab-drag labels and validation messages requested by the maintainer; all 24 catalogs covered; scores and estimates unchanged. |
| 2026-10-22 | minor | Tab-drag follow-up: reconcile upstream progress updates and document drag lifecycle, insertion behavior and regression contracts; behavior, scores and estimates unchanged. |
| 2026-10-22 | minor | Artwork dragged with Selection switches visible document tabs after a 400 ms hover, then copies editable artwork on canvas release; source and clipboard stay intact; scores and hours unchanged |

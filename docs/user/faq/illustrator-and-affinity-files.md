---
title: Can VectorCraft open Illustrator and Affinity files?
description: VectorCraft opens Illustrator .ai files and Affinity files, with limits, and saves .ai only in the PDF-compatible form.
order: 20
verified: 0.9.0
---

Yes to both, with limits. VectorCraft opens Illustrator `.ai` files and Affinity files. It writes `.ai` only as a PDF-compatible file, and it does not write Affinity files.

## Illustrator (.ai)

- VectorCraft reads the PDF page inside an `.ai` file, including vectors, text, images, color, transparency and layers. It also reads the structure of Illustrator's editing data.
- Saving as **PDF-compatible .ai** writes a file that Illustrator opens as plain artwork, without Illustrator's private editing data.
- Known problems with real files: the layer structure can come out flattened or multiplied, guides can be lost, and live effects, brushes and symbols can come in as their drawn look.

## Affinity

VectorCraft opens `.af` (Affinity 3), `.afdesign` and `.afpub` files, and `.afphoto` files from Affinity 1 and 2 by their content. What could not come in is listed in the warning when the file opens.

## More

See [Open a file](../guides/open-a-file.md). If a file of yours does not open right, [report the problem](../troubleshooting/bug-reports.md).

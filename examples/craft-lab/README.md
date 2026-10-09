# Fieldwork Studio

An original, dependency-free working specimen for the personal harness. Fictional
materials and planning figures demonstrate UI, motion, editable vector artwork
and document production. This is a reference exercise, not a production service.

## Try the interface

From the harness root:

```sh
python3 -m http.server 4173 --bind 127.0.0.1 --directory examples/craft-lab
```

Open `http://127.0.0.1:4173`. Search materials, filter by category, save an item,
open its preview, close with Escape, view Saved, remove the last saved item,
and recover from a search with no results. Data is an in-memory fictional
collection and resets on reload. Inspect mobile and reduced-motion behavior.
Stop the local server when finished.

## Produce evidence and documents

Create `build` once. Configure existing tools using the
[workbench guide](../../docs/workbench.md), then run:

```sh
python3 ai.py workbench browser --scenario examples/craft-lab/scenario.json --output build/fieldwork-browser
python3 ai.py workbench vector examples/craft-lab/fieldwork.svg --output build/fieldwork-vector
python3 ai.py workbench documents demo --output build/fieldwork-office
python3 ai.py workbench markdown examples/craft-lab/README.md --output build/fieldwork-markdown
```

Use a new output name for another run. Open PNGs and editable originals, not only
the JSON reports. The browser scenario checks one search/filter/save/preview
journey across four viewport/motion combinations; it is not exhaustive UI QA.
The standalone botanical notebook illustration is intended for 64px and larger;
small icon use needs simplified artwork. The UI contains six other original SVG
studies with distinct materials and editable source in `index.html`.

The office demo is a website-refresh proposal: editable DOCX/PPTX/XLSX with
$11,600 base cost, $1,160 contingency and $12,760 total. It demonstrates a narrow
render/recalculation path; imported complex Office files need their own checks.

## Design intent

Calm warm surfaces, clear working hierarchy, restrained green feedback, native
controls, content-led composition and consistent original illustration. User
references informed clarity and interaction expectations; no brand UI or assets
were copied. A pleasing specimen is not measured evidence of agent improvement.

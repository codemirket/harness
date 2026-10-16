# Uniform native-pixel inspection tiles

Use `tile-judge-views.py` only on **new, unscored** packets, after the normal
packet builder and before either main judge receives the packet. It makes no
model calls and changes no app, observer, action plan, brief, rubric or score
format. Existing overview PNGs and every original view remain unchanged.

For every available `fullpage` PNG with height **greater than 2048 pixels**, append
native-pixel vertical tiles with full original width, maximum height **1536**, and
**128 pixels of overlap**. Start at row zero, advance by 1408 rows and include the
last row. The final tile may be shorter. No resizing, content selection, preferred
frames, width/state/arm exception or hook-specific selection occurs. Smaller
fullpages, unavailable views and viewport views remain as originally built.

Apply one setup-level operation to both A/B packets and their private index:

```sh
python3 evals/design-quality/tile-judge-views.py apply \
  --setup NEW_EXPERIMENT/judge-packet-setup --update-new-unscored
python3 evals/design-quality/tile-judge-views.py verify \
  --setup NEW_EXPERIMENT/judge-packet-setup
```

The explicit update flag authorizes replacing only the two new `packet.json`
files and the new standard `render-identities.private.json`. It does not
authenticate whether somebody privately formed scores. The helper requires
blank score templates, a complete current private file index and no score/guess
artifacts, locks, previous tile outputs, tile manifest or transaction evidence.
All supplied and indexed paths must be normal relative project paths without
symlink components. Both packet directories must have the existing neutral
packet schema. No previous study is modified automatically.

Public packets append neutral `detail` views after their original view list,
ordered by original artifact/view order and then top-to-bottom tile order. Each
has a unique ID, `availability: available`, a neutral PNG path, the source view's
viewport/state and a `tile` rectangle `{x,y,width,height}`. They contain no hashes,
byte sizes, source identities or arm strings. Inspect every available detail at
readable native pixel resolution alongside its complete overview. Tiles are
additional views under the same rubric, not extra score dimensions or outcomes.

Private files remain outside both read packets:

- `render-identities.pre-tiles.private.json` and `packets.pre-tiles.private/`
  preserve the builder's exact original index and packet JSON bytes.
- The final standard index retains all original source/view/file records,
  updates packet-JSON bindings and appends every tile binding and provenance.
  The unchanged score-lock helper therefore verifies all current packet files,
  including the new PNGs, and accepts located evidence naming available details.
- `tile-manifest.private.json` binds original/final packets and index, original
  PNGs, each exact crop rectangle, tile PNGs and decoded every-channel pixel
  equality. Original PNG bytes are rechecked and never rewritten.
- `tile-transaction.private.json` records progress and final verification. Any
  partial failure stops with backups, staging and the journal preserved for
  reconciliation. Applying again refuses these files; it never silently replays
  or repairs a partially transformed setup.

Freeze the final packet/index/manifest hashes before releasing either packet.
Keep every private file and source mapping away from judges until score lock.
The original builder's private index remains historical lineage, not a current
index to pass to score locking after transformation.

Safety bounds are 64 million decoded pixels and 64 MiB per PNG, two billion
cumulative source pixels, 64 tiles per source view, 8192 total tiles and 4 GiB
total encoded tile data. Inputs exceeding bounds fail explicitly. Palette PNGs
decode to explicit RGBA for lossless displayed-color/alpha comparison; other
supported PNG modes retain their native samples. No dependencies are installed.

Run the original non-model synthetic checks with:

```sh
python3 evals/design-quality/test_tile_judge_views.py
```

Uniform tiles cover vertical reading regions in all originally available fixed
fullpage states. They do not reveal unseen horizontal or dynamic offviewport
behavior, guarantee that a judge inspected every pixel, or establish semantic
correctness, human acceptance or assistive-technology behavior. Hashes and file
checks establish identity; they are not an execution sandbox or judge blinding
proof.

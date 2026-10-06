# Document the local record filter

Work only inside the supplied scratch `workspace/`. The Python command and CSV are the fixture sources. Do not use external docs or add dependencies.

Write `README.md` for someone who must use `mini_lookup.py` without reading its implementation. Explain the accepted arguments, matching and output behavior, error behavior, and how to run it from this directory. Include at least two fenced `bash` examples that actually run against `records.csv`, and describe what the reader should expect. Cite the supplied script and data file with relative Markdown links next to source-derived claims. Run the examples before finishing. Correct the tool only if needed to make the documented contract truthful, and explain any such change.

You may verify the examples locally with `python3 -I ../verify.py .` from this workspace. A reviewer still needs to check prose accuracy.

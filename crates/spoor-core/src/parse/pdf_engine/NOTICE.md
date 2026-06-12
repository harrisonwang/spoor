This directory is derived from `pdf-extract` 0.7.12 by Jeff Muizelaar:
https://github.com/jrmuizel/pdf-extract

The upstream crate is distributed under the MIT License. spoor retains the
in-memory extraction implementation and replaces direct stdout/stderr
diagnostics with a no-op logging macro so the embedding core has no process
stream side effects.

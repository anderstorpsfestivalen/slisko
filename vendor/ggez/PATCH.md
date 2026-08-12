# Local ggez patch

This directory vendors `ggez` 0.10.0 from crates.io.

The upstream `GraphicsContext::begin_frame` retries
`CurrentSurfaceTexture::Timeout` and `CurrentSurfaceTexture::Occluded` in a
tight loop. On macOS, an initially covered window can remain occluded until
AppKit processes more events, so retrying on the main thread prevents the
condition from ever clearing.

The local patch returns a `FrameAcquireStatus` for those recoverable outcomes.
Callers can then return control to the event loop and retry when appropriate.
Remove the `[patch.crates-io]` entry and this directory once upstream ggez
supports skipping unavailable surface frames.

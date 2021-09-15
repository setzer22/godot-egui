# Frequently asked questions.

## Why do my egui widgets only partially render?

Aside from issues that may be caused by `egui` itself.

Possible causes in `GodotEgui`:

### Batch Buffer Size is too small

Check your console log for the following error "WARNING: _prefill_polygon: poly has too many indices to draw, increase batch buffer size. At: ./drivers/gles_common/rasterizer_canvas_batcher.h:1628"

If this is the case, in your Godot's "Project Settings" go to "Rendering -> Batching" and set the batch buffer size to 65535. While it is possible to be lower, egui expects to be able to use at least a 16bit indice value for the GUI.

### Multiple `GodotEgui` contexts are overlapping.

`egui` is designed with the assumption that each `egui::Context` behaves as an explicity "application" so when using multiple `GodotEgui` nodes it is possible for the overlap between the nodes to occlude nodes on a lower Z Level.

### A bug

If neither of the bugs above occur and your UI renders correctly when using [eframe](https://lib.rs/crates/eframe) or other native egui solution, this may be an issue with `GodotEgui` please feel free to open an issue with a link to a project that can reproduce the issue.
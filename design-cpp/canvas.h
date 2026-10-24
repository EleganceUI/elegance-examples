#ifndef DESIGN_CANVAS_H
#define DESIGN_CANVAS_H

#include "model.h"

#include <elg/elegance.h>

namespace design {

// The infinite canvas: the layers, selection and handles, and everything the pointer and
// keyboard do there (selecting, moving, resizing, drawing, panning, zooming).
elg::View CanvasView(const Document& doc, elg::State<elg::Size> viewport);

} // namespace design

#endif // DESIGN_CANVAS_H

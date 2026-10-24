// WebGPU: a spinning cube drawn by wgpu on its own thread, shown in an Elegance window with
// controls over it. Frames are rendered offscreen and read back into a VideoFeed on every
// desktop platform. Drag the cube to turn it, scroll to zoom: an event handler
// on the image sends the window's events to the drawing thread.

use std::process::ExitCode;

use elegance::prelude::*;

mod renderer;

const SIZE: u32 = 512; // pixels, square

// The cube, filling the window, with a speed slider over it.
struct Cube {
    image: renderer::Image,
    renderer: std::rc::Rc<renderer::CubeRenderer>,
}

// Sends pointer drags and scrolling over the image to the renderer.
struct CubeInput(std::sync::mpsc::Sender<renderer::Input>);

impl EventModifier for CubeInput {
    fn make_handler(&self) -> Box<dyn EventHandler> {
        Box::new(CubeHandler {
            input: self.0.clone(),
            last: None,
        })
    }
}

struct CubeHandler {
    input: std::sync::mpsc::Sender<renderer::Input>,
    last: Option<Point>, // while dragging
}

impl EventHandler for CubeHandler {
    fn pointer(&mut self, event: &PointerEvent, _frame: Rect) -> EventResult {
        use renderer::Input;
        match event.kind {
            PointerEventKind::Down if event.button == MouseButton::Left => {
                self.last = Some(event.location);
                let _ = self.input.send(Input::Grab(true));
                EventResult::CONSUMED
            }
            PointerEventKind::Move => {
                let Some(last) = self.last.replace(event.location) else {
                    return EventResult::IGNORED;
                };
                let (dx, dy) = (event.location.x - last.x, event.location.y - last.y);
                let _ = self.input.send(Input::Drag { dx, dy });
                EventResult::CONSUMED
            }
            PointerEventKind::Up | PointerEventKind::Cancel if self.last.take().is_some() => {
                let _ = self.input.send(Input::Grab(false));
                EventResult::CONSUMED
            }
            _ => EventResult::IGNORED,
        }
    }

    fn scroll(&mut self, event: &ScrollEvent) -> EventResult {
        // In points, 48 a wheel notch, positive scrolling down: up zooms in.
        let _ = self
            .input
            .send(renderer::Input::Zoom(1.1f32.powf(-event.dy / 48.0)));
        EventResult::CONSUMED
    }

    // The whole image takes events, not only where something is drawn.
    fn covers_frame(&self) -> bool {
        true
    }
}

impl Component for Cube {
    fn body(&self) -> impl IntoView {
        let speed = State::new(60.0f32);
        let renderer = self.renderer.clone();
        let binding = Binding::new(
            move || speed.get(),
            move |value| {
                speed.set(value);
                renderer.set_speed(value);
            },
        );
        elg! {
            ZStack(alignment: .bottom) {
                { self.image.view() }
                    .modifier(AsEvent(CubeInput(self.renderer.input())))
                HStack(spacing: 12) {
                    Text("Speed")
                    Slider(binding, min: -360, max: 360).frame(width: 220)
                    Text(move || format!("{}°/s", speed.get() as i32)).frame(width: 64)
                }
                .padding(12)
                .background_in(Material::REGULAR, Capsule())
                .padding(20)
            }
        }
    }
}

fn main() -> Result<ExitCode, Error> {
    let app = App::new();
    let image = renderer::Image::new(SIZE)?;
    let renderer = std::rc::Rc::new(renderer::CubeRenderer::new(image.clone(), 60.0));
    let result = app
        .window(
            WindowDesc::new("WebGPU").size(640, 640),
            Cube {
                image,
                renderer: renderer.clone(),
            },
        )
        .run();
    renderer.stop(); // before the app goes
    result
}

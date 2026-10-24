// Pure-Rust direct GPU cube: drag to rotate, scroll to zoom.

use elegance::prelude::*;
use std::{process::ExitCode, rc::Rc};

mod renderer;
use renderer::{CubeRenderer, Input};

struct Cube {
    renderer: Rc<CubeRenderer>,
}

impl Cube {
    fn new() -> Result<Self, Error> {
        Ok(Self {
            renderer: Rc::new(CubeRenderer::new(512, 60.0)?),
        })
    }
}

impl Component for Cube {
    fn body(&self) -> impl IntoView {
        let speed = State::new(60.0f32);

        // The mounted effect owns the renderer, not just its input sender: component
        // descriptions can be dropped after body() builds their element tree.
        let renderer = self.renderer.clone();
        effect(move || {
            let _ = renderer.input.send(Input::Speed(speed.get()));
        });

        let pointer = self.renderer.input.clone();
        let scroll = self.renderer.input.clone();
        elg! {
            ZStack(alignment: .bottom) {
                ExternalGpuImageView(&self.renderer.image, mode: .fill)
                    .on_drag(move |event| {
                        let input = match event {
                            DragEvent::Started => Input::Grab(true),
                            DragEvent::Moved { delta } => Input::Drag { dx: delta.x, dy: delta.y },
                            DragEvent::Ended => Input::Grab(false),
                        };
                        let _ = pointer.send(input);
                    })
                    .on_scroll(move |event| {
                        let _ = scroll.send(Input::Zoom(1.1f32.powf(-event.dy / 48.0)));
                        true
                    })
                HStack(spacing: 12) {
                    Text("Speed")
                    Slider($speed, min: -360, max: 360).frame(width: 220)
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
    App::new()
        .window(
            WindowDesc::new("WebGPU · Direct GPU").size(640, 640),
            Cube::new()?,
        )
        .run()
}

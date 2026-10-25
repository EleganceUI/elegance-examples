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
            renderer: Rc::new(CubeRenderer::new(60.0)?),
        })
    }
}

#[component]
fn InfoCard(
    title: String,
    #[prop(default = HorizontalAlignment::Leading)] alignment: elegance::HorizontalAlignment,
    #[prop(default = Weight::Regular)] weight: elegance::Weight,
) -> impl IntoView {
    elg! {
        VStack(spacing: 8, alignment: alignment) {
            Text(title.clone())
                .font_weight(weight)

            Text("This is a custom Rust component.")
                .foreground_style(.secondary)
        }
        .padding(16)
    }
}

impl Component for Cube {
    fn body(&self) -> impl IntoView {
        let speed = State::new(60.0f32);
        let renderer = self.renderer.clone();

        effect(move || {
            let _ = renderer.input.send(Input::Speed(speed.get()));
        });

        let pointer = self.renderer.input.clone();
        let scroll = self.renderer.input.clone();
        let resize = self.renderer.input.clone();
        elg! {
            VStack {
                Text("Hello, World", padding: 24.)
                ZStack(alignment: .bottom) {
                    GpuImageView(&self.renderer.image, mode: .fill)
                        .on_sized(move |size| {
                            let _ = resize.send(Input::Resize(size));
                        })
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
                        .corner_radius(24.)
                    VStack(spacing: 12., alignment: .leading) {
                        HStack {
                            Text("Hello, World")
                            Button(action: || println!("Pressed!")) {
                                Text("Press me!")
                            }
                        }
                        HStack(spacing: 12) {
                            Text("Speed")
                            Slider($speed, min: -360, max: 360).frame(width: 220)
                            Text(move || format!("{}°/s", speed.get() as i32)).frame(width: 64)
                        }
                        .padding(8)
                        .background_in(Material::ULTRA_THICK, RoundedRectangle(CornerRadii::all(8.)))
                    }
                    .padding(12)
                    .background(Material::REGULAR)
                    .corner_radius(12.)
                    .padding(24)
                }.padding(12.)
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

// Pure-Rust CPU-readback cube: drag to rotate, scroll to zoom.

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

impl Component for Cube {
    fn body(&self) -> impl IntoView {
        let speed = State::new(60.0f32);
        let render_size = State::new((0, 0));
        let renderer = self.renderer.clone();

        effect(move || {
            let _ = renderer.input.send(Input::Speed(speed.get()));
        });

        let pointer = self.renderer.input.clone();
        let scroll = self.renderer.input.clone();
        let resize = self.renderer.input.clone();

        elg! {
            VStack(spacing: 12) {
                Text("Rendering at {render_size.0}x{render_size.1}.")
                ZStack(alignment: .bottom) {
                    VideoFeedView(&self.renderer.image, mode: .fill)
                        .on_sized(move |size| {
                            render_size.set((size.width, size.height));
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
                        .corner_radius(12)
                    VStack(spacing: 12., alignment: .leading) {
                        HStack {
                            Text("Hello, World")
                            Button(label: "Press me!", action: || println!("Pressed!"))
                        }
                        HStack(spacing: 12) {
                            Text("Speed")
                            Slider(speed, min: -360, max: 360).frame(width: 220)
                            Text(move || format!("{}°/s", speed.get() as i32)).frame(width: 64)
                        }
                        .padding(8)
                        .background(Material::ULTRA_THICK)
                        .clip_shape(RoundedRectangle(CornerRadii::all(8.)))
                    }
                    .padding(12)
                    .background(Material::REGULAR)
                    .corner_radius(12.)
                    .padding(12)
                }
            }.padding(12)
        }
    }
}

fn main() -> Result<ExitCode, Error> {
    App::new()
        .window(
            WindowDesc::new("wgpu · Image (CPU copy)")
                .size(640, 640)
                .title_bar(TitleBar::Integrated)
                .traffic_lights_position(14., 14.),
            Cube::new()?,
        )
        .run()
}

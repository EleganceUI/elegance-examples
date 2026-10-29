// Pure-Rust CPU-readback cube: drag to rotate, scroll to zoom.

use elegance::prelude::*;
use std::{cell::Cell, process::ExitCode, rc::Rc};

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

        // A drag reports how far it's gone since the press; the renderer wants each step.
        let last = Rc::new(Cell::new(Point::default()));
        let drag = DragGesture::new()
            .on_began({
                let (input, last) = (self.renderer.input.clone(), last.clone());
                move |event| {
                    last.set(event.translation);
                    let _ = input.send(Input::Grab(true));
                }
            })
            .on_changed({
                let input = self.renderer.input.clone();
                move |event| {
                    let previous = last.replace(event.translation);
                    let (dx, dy) = (event.translation.x - previous.x, event.translation.y - previous.y);
                    let _ = input.send(Input::Drag { dx, dy });
                }
            })
            .on_ended({
                let input = self.renderer.input.clone();
                move |_| {
                    let _ = input.send(Input::Grab(false));
                }
            });
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
                        .recognize(drag)
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
                        .background(.glass_ultra_thick)
                        .backdrop_blur(.xxl, saturation: 1.8, noise: 0.02)
                        .clip_shape(RoundedRectangle(CornerRadii::all(8.)))
                    }
                    .padding(12)
                    .background(.glass_regular)
                    .backdrop_blur(.xl, saturation: 1.8, noise: 0.02)
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
                .system_controls_position(14., 14.),
            Cube::new()?,
        )
        .run()
}

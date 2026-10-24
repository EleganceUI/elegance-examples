// WebGPU: a spinning cube drawn by wgpu on its own thread, shown in an Elegance window with
// controls over it. wgpu draws straight into a GpuImage's shared textures (no copies) and
// signals its shared fence when each frame is done. Windows only for now, like GpuImage.

use std::process::ExitCode;

use elegance::prelude::*;

#[cfg(windows)]
mod renderer;

const SIZE: u32 = 512; // pixels, square

// The cube, filling the window, with a speed slider over it.
#[cfg(windows)]
struct Cube {
    image: GpuImage,
    renderer: std::rc::Rc<renderer::CubeRenderer>,
}

#[cfg(windows)]
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
                GpuImageView(&self.image).mode(ContentMode::Fill)
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

#[cfg(windows)]
fn main() -> Result<ExitCode, Error> {
    let app = App::new();
    let image = GpuImage::new(SIZE, SIZE, GpuImageOrigin::TopLeft)?;
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

#[cfg(not(windows))]
fn main() -> Result<ExitCode, Error> {
    let _ = SIZE;
    App::new()
        .window(WindowDesc::new("WebGPU").size(480, 200), || {
            elg! { Text("The webgpu example runs on Windows only for now").padding(24) }
        })
        .run()
}

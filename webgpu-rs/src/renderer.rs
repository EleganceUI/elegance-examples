// The cube, drawn into an offscreen wgpu texture on its own thread. Frames are read back
// as BGRA pixels for Elegance's VideoFeed, using the same path on every desktop platform.

use std::future::Future;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{mpsc, Arc, Mutex};
use std::task::{Context, Poll, Waker};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use elegance::{VideoFeed, VideoFormat};
use wgpu::util::DeviceExt;

const SAMPLES: u32 = 4;
const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth24Plus;

const COLOR_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Bgra8Unorm;

/// The platform's presentation path, shared by the UI and render thread.
#[derive(Clone)]
pub struct Image {
    feed: VideoFeed,
    size: u32,
}

impl Image {
    pub fn new(size: u32) -> Result<Self, elegance::Error> {
        Ok(Self {
            feed: VideoFeed::new()?,
            size,
        })
    }

    pub fn view(&self) -> impl elegance::IntoView {
        elegance::VideoFeedView::new(&self.feed).mode(elegance::ContentMode::Fill)
    }
}

/// What the window's events do to the cube, sent to its thread.
pub enum Input {
    /// The pointer went down on the cube (true), or let go (false). It stops spinning while held.
    Grab(bool),
    /// Turn it by a drag, in points.
    Drag { dx: f32, dy: f32 },
    /// Move it closer (above 1) or farther (below 1).
    Zoom(f32),
}

/// Draws the cube into `image` until stopped or dropped.
pub struct CubeRenderer {
    speed: Arc<AtomicU32>, // degrees per second, as f32 bits
    input: Sender<Input>,
    running: Arc<AtomicBool>,
    thread: Mutex<Option<JoinHandle<()>>>,
}

impl CubeRenderer {
    pub fn new(image: Image, speed: f32) -> Self {
        let speed = Arc::new(AtomicU32::new(speed.to_bits()));
        let running = Arc::new(AtomicBool::new(true));
        let (input, inputs) = mpsc::channel();
        let thread = std::thread::spawn({
            let (speed, running) = (speed.clone(), running.clone());
            move || {
                if let Err(message) = run(&image, &speed, &inputs, &running) {
                    eprintln!("webgpu: {message}");
                }
            }
        });
        Self {
            speed,
            input,
            running,
            thread: Mutex::new(Some(thread)),
        }
    }

    /// Degrees per second.
    pub fn set_speed(&self, speed: f32) {
        self.speed.store(speed.to_bits(), Ordering::Relaxed);
    }

    /// Where to send the window's events (from any thread).
    pub fn input(&self) -> Sender<Input> {
        self.input.clone()
    }

    /// Stops drawing, once the thread has finished its frame.
    pub fn stop(&self) {
        self.running.store(false, Ordering::Relaxed);
        if let Some(thread) = self.thread.lock().unwrap().take() {
            let _ = thread.join();
        }
    }
}

impl Drop for CubeRenderer {
    fn drop(&mut self) {
        self.stop();
    }
}

// The offscreen render target.
struct Target {
    view: wgpu::TextureView,
    texture: wgpu::Texture,
}

// Where the cube is turned to, and how far away it is.
struct Pose {
    yaw: f32,   // degrees
    pitch: f32, // degrees
    distance: f32,
    held: bool,
}

impl Pose {
    fn apply(&mut self, input: Input) {
        match input {
            Input::Grab(held) => self.held = held,
            Input::Drag { dx, dy } => {
                self.yaw += dx * 0.5;
                self.pitch += dy * 0.5;
            }
            Input::Zoom(factor) => self.distance = (self.distance / factor).clamp(3.5, 15.0),
        }
    }
}

fn run(
    image: &Image,
    speed: &AtomicU32,
    inputs: &Receiver<Input>,
    running: &AtomicBool,
) -> Result<(), String> {
    let gpu = Gpu::new(image.size, image.size)?;
    let texture = gpu.device.create_texture(&wgpu::TextureDescriptor {
        label: Some("cube output"),
        size: wgpu::Extent3d {
            width: image.size,
            height: image.size,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: COLOR_FORMAT,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });
    let target = Target {
        view: texture.create_view(&Default::default()),
        texture,
    };
    // Texture-to-buffer copies require padded rows; VideoFeed takes tightly packed BGRA.
    let row_bytes = image.size * 4;
    let padded_row_bytes =
        row_bytes.div_ceil(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT) * wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
    let readback = gpu.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("cube readback"),
        size: u64::from(padded_row_bytes) * u64::from(image.size),
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    let mut pixels = vec![0; row_bytes as usize * image.size as usize];
    let mut pose = Pose {
        yaw: 0.0,
        pitch: 0.0,
        distance: 6.0,
        held: false,
    };
    let mut last = Instant::now();
    while running.load(Ordering::Relaxed) {
        let now = Instant::now();
        while let Ok(input) = inputs.try_recv() {
            pose.apply(input);
        }
        if !pose.held {
            let turn = f32::from_bits(speed.load(Ordering::Relaxed)) * (now - last).as_secs_f32();
            pose.yaw += turn;
            pose.pitch += turn * 0.7;
        }
        last = now;
        let mut encoder = gpu.draw(&target, &pose);
        encoder.copy_texture_to_buffer(
            target.texture.as_image_copy(),
            wgpu::TexelCopyBufferInfo {
                buffer: &readback,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(padded_row_bytes),
                    rows_per_image: None,
                },
            },
            target.texture.size(),
        );
        let submission = gpu.queue.submit([encoder.finish()]);
        let (tx, rx) = mpsc::channel();
        readback
            .slice(..)
            .map_async(wgpu::MapMode::Read, move |result| {
                let _ = tx.send(result);
            });
        gpu.device
            .poll(wgpu::PollType::Wait {
                submission_index: Some(submission),
                timeout: Some(Duration::from_secs(5)),
            })
            .map_err(|error| format!("GPU readback failed: {error}"))?;
        rx.recv_timeout(Duration::from_secs(5))
            .map_err(|error| error.to_string())?
            .map_err(|error| error.to_string())?;
        {
            let mapped = readback
                            .slice(..)
                            .get_mapped_range()
                            .map_err(|error| format!("GPU mapped range failed: {error}"))?;
            for (src, dst) in mapped
                .chunks_exact(padded_row_bytes as usize)
                .zip(pixels.chunks_exact_mut(row_bytes as usize))
            {
                dst.copy_from_slice(&src[..row_bytes as usize]);
            }
        }
        readback.unmap();
        image
            .feed
            .push(VideoFormat::Bgra, image.size, image.size, &pixels, true);
        std::thread::sleep(
            (now + Duration::from_micros(16_667)).saturating_duration_since(Instant::now()),
        );
    }
    Ok(())
}

struct Gpu {
    device: wgpu::Device,
    queue: wgpu::Queue,
    pipeline: wgpu::RenderPipeline,
    vertices: wgpu::Buffer,
    uniforms: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    samples: wgpu::TextureView, // drawn multisampled, resolved into the image's buffer
    depth: wgpu::TextureView,
}

impl Gpu {
    fn new(width: u32, height: u32) -> Result<Self, String> {
        let adapter = block_on(
            wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle())
                .request_adapter(&wgpu::RequestAdapterOptions::default()),
        )
        .map_err(|error| format!("can't find a GPU adapter: {error}"))?;
        Self::with_adapter(adapter, width, height)
    }

    fn with_adapter(adapter: wgpu::Adapter, width: u32, height: u32) -> Result<Self, String> {
        let (device, queue) = block_on(adapter.request_device(&wgpu::DeviceDescriptor {
            label: Some("cube"),
            ..Default::default()
        }))
        .map_err(|error| error.to_string())?;

        let shader = device.create_shader_module(wgpu::include_wgsl!("cube.wgsl"));
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("cube"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("cube"),
            layout: Some(
                &device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("cube"),
                    bind_group_layouts: &[Some(&layout)],
                    immediate_size: 0,
                }),
            ),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                compilation_options: Default::default(),
                buffers: &[Some(wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<[f32; 6]>() as u64,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x3],
                })],
            },
            primitive: wgpu::PrimitiveState {
                cull_mode: Some(wgpu::Face::Back),
                ..Default::default()
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: DEPTH_FORMAT,
                depth_write_enabled: Some(true),
                depth_compare: Some(wgpu::CompareFunction::Less),
                stencil: Default::default(),
                bias: Default::default(),
            }),
            multisample: wgpu::MultisampleState {
                count: SAMPLES,
                ..Default::default()
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                compilation_options: Default::default(),
                targets: &[Some(COLOR_FORMAT.into())],
            }),
            multiview_mask: None,
            cache: None,
        });

        let vertices = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("cube"),
            contents: as_bytes(&cube_vertices()),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let uniforms = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("cube"),
            size: std::mem::size_of::<[f32; 16]>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("cube"),
            layout: &layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniforms.as_entire_binding(),
            }],
        });

        let attachment = |format| {
            device
                .create_texture(&wgpu::TextureDescriptor {
                    label: Some("cube"),
                    size: wgpu::Extent3d {
                        width,
                        height,
                        depth_or_array_layers: 1,
                    },
                    mip_level_count: 1,
                    sample_count: SAMPLES,
                    dimension: wgpu::TextureDimension::D2,
                    format,
                    usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                    view_formats: &[],
                })
                .create_view(&Default::default())
        };
        let samples = attachment(COLOR_FORMAT);
        let depth = attachment(DEPTH_FORMAT);

        Ok(Self {
            device,
            queue,
            pipeline,
            vertices,
            uniforms,
            bind_group,
            samples,
            depth,
        })
    }

    fn draw(&self, target: &Target, pose: &Pose) -> wgpu::CommandEncoder {
        let aspect = target.texture.width() as f32 / target.texture.height() as f32;
        let transform = mul(
            perspective(aspect),
            mul(
                translation(0.0, 0.0, -pose.distance),
                mul(rotation_x(pose.pitch), rotation_y(pose.yaw)),
            ),
        );
        self.queue
            .write_buffer(&self.uniforms, 0, as_bytes(&transform));

        let mut encoder = self.device.create_command_encoder(&Default::default());
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("cube"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &self.samples,
                    depth_slice: None,
                    resolve_target: Some(&target.view),
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.07,
                            g: 0.08,
                            b: 0.11,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Discard,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Discard,
                    }),
                    stencil_ops: None,
                }),
                ..Default::default()
            });
            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, &self.bind_group, &[]);
            pass.set_vertex_buffer(0, self.vertices.slice(..));
            pass.draw(0..36, 0..1);
        }
        encoder
    }
}

// Each face: a color, then its four corners (counterclockwise from outside), as two triangles.
fn cube_vertices() -> Vec<[f32; 6]> {
    #[rustfmt::skip]
    const FACES: [([f32; 3], [[f32; 3]; 4]); 6] = [
        ([1.00, 0.37, 0.43], [[-1., -1., 1.], [1., -1., 1.], [1., 1., 1.], [-1., 1., 1.]]),     // front
        ([0.28, 0.81, 0.45], [[1., -1., -1.], [-1., -1., -1.], [-1., 1., -1.], [1., 1., -1.]]), // back
        ([0.24, 0.65, 1.00], [[-1., 1., 1.], [1., 1., 1.], [1., 1., -1.], [-1., 1., -1.]]),     // top
        ([1.00, 0.76, 0.44], [[-1., -1., -1.], [1., -1., -1.], [1., -1., 1.], [-1., -1., 1.]]), // bottom
        ([0.64, 0.42, 1.00], [[1., -1., 1.], [1., -1., -1.], [1., 1., -1.], [1., 1., 1.]]),     // right
        ([0.95, 0.95, 0.97], [[-1., -1., -1.], [-1., -1., 1.], [-1., 1., 1.], [-1., 1., -1.]]), // left
    ];
    let mut vertices = Vec::with_capacity(36);
    for (color, corners) in FACES {
        for i in [0, 1, 2, 0, 2, 3] {
            // A little darker toward two corners, so the faces read as surfaces.
            let shade = if i < 2 { 0.78 } else { 1.0 };
            let [x, y, z] = corners[i];
            vertices.push([
                x,
                y,
                z,
                color[0] * shade,
                color[1] * shade,
                color[2] * shade,
            ]);
        }
    }
    vertices
}

// 4x4 matrices, column-major like WGSL's.
type Mat4 = [f32; 16];

fn mul(a: Mat4, b: Mat4) -> Mat4 {
    let mut out = [0.0; 16];
    for column in 0..4 {
        for row in 0..4 {
            out[column * 4 + row] = (0..4).map(|k| a[k * 4 + row] * b[column * 4 + k]).sum();
        }
    }
    out
}

// The same frustum as the OpenGL example, with WebGPU's depth range (0 to 1).
fn perspective(aspect: f32) -> Mat4 {
    let (near, far) = (1.0f32, 20.0f32);
    let (x, y) = (2.0 * near / aspect.max(1.0), 2.0 * near * aspect.min(1.0));
    #[rustfmt::skip]
    let m = [
        x, 0.0, 0.0, 0.0,
        0.0, y, 0.0, 0.0,
        0.0, 0.0, far / (near - far), -1.0,
        0.0, 0.0, near * far / (near - far), 0.0,
    ];
    m
}

fn translation(x: f32, y: f32, z: f32) -> Mat4 {
    [
        1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, x, y, z, 1.0,
    ]
}

fn rotation_x(degrees: f32) -> Mat4 {
    let (s, c) = degrees.to_radians().sin_cos();
    [
        1.0, 0.0, 0.0, 0.0, 0.0, c, s, 0.0, 0.0, -s, c, 0.0, 0.0, 0.0, 0.0, 1.0,
    ]
}

fn rotation_y(degrees: f32) -> Mat4 {
    let (s, c) = degrees.to_radians().sin_cos();
    [
        c, 0.0, -s, 0.0, 0.0, 1.0, 0.0, 0.0, s, 0.0, c, 0.0, 0.0, 0.0, 0.0, 1.0,
    ]
}

fn as_bytes<T: Copy>(values: &[T]) -> &[u8] {
    // f32 arrays only: no padding.
    unsafe { std::slice::from_raw_parts(values.as_ptr().cast(), std::mem::size_of_val(values)) }
}

// wgpu's native futures are ready at once (or after a moment): poll until done.
fn block_on<F: Future>(future: F) -> F::Output {
    let mut future = std::pin::pin!(future);
    let mut context = Context::from_waker(Waker::noop());
    loop {
        if let Poll::Ready(output) = future.as_mut().poll(&mut context) {
            return output;
        }
        std::thread::yield_now();
    }
}

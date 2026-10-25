// The cube, drawn into an offscreen wgpu texture on its own thread. Frames are read back
// as BGRA pixels for Elegance's VideoFeed, using the same path on every desktop platform.

use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

use elegance::{Error, ErrorKind, PixelSize, VideoFeed, VideoFormat};
use pollster::block_on;
use wgpu::util::DeviceExt;

const SAMPLES: u32 = 4;
const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth24Plus;
const COLOR_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Bgra8Unorm;

pub enum Input {
    Resize(PixelSize),
    Speed(f32),
    Grab(bool),
    Drag { dx: f32, dy: f32 },
    Zoom(f32),
    Stop,
}

pub struct CubeRenderer {
    pub image: VideoFeed,
    pub input: mpsc::Sender<Input>,
    producer: Option<thread::JoinHandle<()>>,
}

impl CubeRenderer {
    pub fn new(speed: f32) -> Result<Self, Error> {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle());
        let adapter = block_on(instance.request_adapter(&Default::default())).map_err(|error| {
            Error::new(
                ErrorKind::Gpu,
                format!("GPU adapter request failed: {error}"),
            )
        })?;
        let (device, queue) = block_on(adapter.request_device(&wgpu::DeviceDescriptor {
            label: Some("cube"),
            ..Default::default()
        }))
        .map_err(|error| {
            Error::new(
                ErrorKind::Gpu,
                format!("GPU device request failed: {error}"),
            )
        })?;
        let image = VideoFeed::new()?;
        let output = image.clone();
        let (input, inputs) = mpsc::channel();
        let producer = thread::spawn(move || {
            if let Err(error) = run(Gpu::new(device, queue), output, speed, inputs) {
                eprintln!("wgpu-image: {error}");
            }
        });
        Ok(Self {
            image,
            input,
            producer: Some(producer),
        })
    }
}

// Stop the producer before the app exits: GPU work still running then races the driver's teardown.
impl Drop for CubeRenderer {
    fn drop(&mut self) {
        let _ = self.input.send(Input::Stop);
        if let Some(producer) = self.producer.take() {
            let _ = producer.join();
        }
    }
}

struct Pose {
    yaw: f32,
    pitch: f32,
    distance: f32,
    speed: f32,
    held: bool,
}

fn run(
    mut gpu: Gpu,
    image: VideoFeed,
    speed: f32,
    inputs: mpsc::Receiver<Input>,
) -> Result<(), String> {
    let mut size = PixelSize::default();
    let mut pose = Pose {
        yaw: 0.0,
        pitch: 0.0,
        distance: 6.0,
        speed,
        held: false,
    };
    let mut last = Instant::now();
    loop {
        loop {
            match inputs.try_recv() {
                Ok(input) => match input {
                    Input::Resize(next) => size = next,
                    Input::Speed(speed) => pose.speed = speed,
                    Input::Grab(held) => pose.held = held,
                    Input::Drag { dx, dy } => {
                        pose.yaw += dx * 0.5;
                        pose.pitch += dy * 0.5;
                    }
                    Input::Zoom(factor) => {
                        pose.distance = (pose.distance / factor).clamp(3.5, 15.0)
                    }
                    Input::Stop => return Ok(()),
                },
                Err(mpsc::TryRecvError::Empty) => break,
                // The mounted UI owns the senders; dropping it stops the producer.
                Err(mpsc::TryRecvError::Disconnected) => return Ok(()),
            }
        }
        let now = Instant::now();
        if size.width == 0 || size.height == 0 {
            last = now;
            thread::sleep(Duration::from_micros(16_667));
            continue;
        }
        if !pose.held {
            let turn = pose.speed * (now - last).as_secs_f32();
            pose.yaw += turn;
            pose.pitch += turn * 0.7;
        }
        last = now;
        gpu.draw(size, &pose, &image)?;
        thread::sleep(
            (now + Duration::from_micros(16_667)).saturating_duration_since(Instant::now()),
        );
    }
}

// All size-dependent resources are replaced together, only when the texture size changes.
struct Target {
    texture: wgpu::Texture,
    view: wgpu::TextureView,
    samples: wgpu::TextureView,
    depth: wgpu::TextureView,
    readback: wgpu::Buffer,
    rows: Rows,
    pixels: Vec<u8>,
}

struct Rows {
    packed: u32,
    padded: u32,
    buffer_size: u64,
    pixel_len: usize,
}

impl Rows {
    fn new(size: PixelSize, limits: &wgpu::Limits) -> Result<Self, String> {
        if size.width == 0 || size.height == 0 {
            return Err("cube target dimensions must be nonzero".into());
        }
        if size.width > limits.max_texture_dimension_2d
            || size.height > limits.max_texture_dimension_2d
        {
            return Err(format!(
                "cube target {}x{} exceeds GPU texture dimension limit {}",
                size.width, size.height, limits.max_texture_dimension_2d
            ));
        }
        let packed = size
            .width
            .checked_mul(4)
            .ok_or("cube row byte count overflow")?;
        let alignment = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
        let padded = packed
            .checked_add(alignment - 1)
            .map(|bytes| bytes / alignment * alignment)
            .ok_or("cube padded row byte count overflow")?;
        let buffer_size = u64::from(padded) * u64::from(size.height);
        if buffer_size > limits.max_buffer_size || buffer_size > isize::MAX as u64 {
            return Err(format!(
                "cube readback requires {buffer_size} bytes, exceeding GPU or host buffer limits"
            ));
        }
        let pixel_len = usize::try_from(u64::from(packed) * u64::from(size.height))
            .map_err(|_| "cube packed pixel byte count exceeds host address space")?;
        Ok(Self {
            packed,
            padded,
            buffer_size,
            pixel_len,
        })
    }
}

impl Target {
    fn new(device: &wgpu::Device, size: PixelSize) -> Result<Self, String> {
        let rows = Rows::new(size, &device.limits())?;
        let mut pixels = Vec::new();
        pixels
            .try_reserve_exact(rows.pixel_len)
            .map_err(|error| format!("cube pixel allocation failed: {error}"))?;
        pixels.resize(rows.pixel_len, 0);
        let texture = |label, format, sample_count, usage| {
            device.create_texture(&wgpu::TextureDescriptor {
                label: Some(label),
                size: wgpu::Extent3d {
                    width: size.width,
                    height: size.height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count,
                dimension: wgpu::TextureDimension::D2,
                format,
                usage,
                view_formats: &[],
            })
        };
        let output = texture(
            "cube output",
            COLOR_FORMAT,
            1,
            wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        );
        let samples = texture(
            "cube MSAA",
            COLOR_FORMAT,
            SAMPLES,
            wgpu::TextureUsages::RENDER_ATTACHMENT,
        )
        .create_view(&Default::default());
        let depth = texture(
            "cube depth",
            DEPTH_FORMAT,
            SAMPLES,
            wgpu::TextureUsages::RENDER_ATTACHMENT,
        )
        .create_view(&Default::default());
        let readback = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("cube readback"),
            size: rows.buffer_size,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        Ok(Self {
            view: output.create_view(&Default::default()),
            texture: output,
            samples,
            depth,
            readback,
            rows,
            pixels,
        })
    }
}

struct Gpu {
    device: wgpu::Device,
    queue: wgpu::Queue,
    pipeline: wgpu::RenderPipeline,
    vertices: wgpu::Buffer,
    uniforms: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    target: Option<Target>,
}

impl Gpu {
    fn new(device: wgpu::Device, queue: wgpu::Queue) -> Self {
        let shader = device.create_shader_module(wgpu::include_wgsl!("cube.wgsl"));

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("cube"),
            layout: None,
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
            layout: &pipeline.get_bind_group_layout(0),
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniforms.as_entire_binding(),
            }],
        });

        Self {
            device,
            queue,
            pipeline,
            vertices,
            uniforms,
            bind_group,
            target: None,
        }
    }

    fn draw(&mut self, size: PixelSize, pose: &Pose, image: &VideoFeed) -> Result<(), String> {
        if self.target.as_ref().is_none_or(|target| {
            target.texture.width() != size.width || target.texture.height() != size.height
        }) {
            self.target = Some(Target::new(&self.device, size)?);
        }
        let target = self.target.as_mut().unwrap();
        let aspect = target.texture.width() as f32 / target.texture.height() as f32;
        let transform = mul(
            perspective(aspect),
            mul(
                translation(0.0, 0.0, -pose.distance),
                mul(rotation_x(pose.pitch), rotation_y(pose.yaw)),
            ),
        );
        self.queue
            .write_buffer(&self.uniforms, 0, as_bytes(&[transform]));

        let mut encoder = self.device.create_command_encoder(&Default::default());
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("cube"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &target.samples,
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
                    view: &target.depth,
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
        encoder.copy_texture_to_buffer(
            target.texture.as_image_copy(),
            wgpu::TexelCopyBufferInfo {
                buffer: &target.readback,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(target.rows.padded),
                    rows_per_image: None,
                },
            },
            target.texture.size(),
        );
        let submission = self.queue.submit([encoder.finish()]);
        let (tx, rx) = mpsc::channel();
        target
            .readback
            .slice(..)
            .map_async(wgpu::MapMode::Read, move |result| {
                let _ = tx.send(result);
            });
        self.device
            .poll(wgpu::PollType::Wait {
                submission_index: Some(submission),
                timeout: Some(Duration::from_secs(5)),
            })
            .map_err(|error| format!("GPU readback wait failed: {error}"))?;
        rx.recv_timeout(Duration::from_secs(5))
            .map_err(|error| format!("GPU readback callback failed: {error}"))?
            .map_err(|error| format!("GPU readback mapping failed: {error}"))?;
        {
            let mapped = target
                .readback
                .slice(..)
                .get_mapped_range()
                .map_err(|error| format!("GPU mapped range failed: {error}"))?;
            for (src, dst) in mapped
                .chunks_exact(target.rows.padded as usize)
                .zip(target.pixels.chunks_exact_mut(target.rows.packed as usize))
            {
                dst.copy_from_slice(&src[..target.rows.packed as usize]);
            }
        }
        target.readback.unmap();
        image.push(
            VideoFormat::Bgra,
            target.texture.width(),
            target.texture.height(),
            &target.pixels,
            true,
        );
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn readback_rows_cover_aligned_and_unaligned_widths() {
        for (width, packed, padded) in [(1, 4, 256), (63, 252, 256), (64, 256, 256), (65, 260, 512)]
        {
            let rows = Rows::new(PixelSize { width, height: 3 }, &wgpu::Limits::default()).unwrap();
            assert_eq!(rows.packed, packed);
            assert_eq!(rows.padded, padded);
            assert_eq!(rows.buffer_size, u64::from(padded) * 3);
            assert_eq!(rows.pixel_len, packed as usize * 3);
        }
    }

    #[test]
    fn readback_rejects_invalid_dimensions_and_buffer_sizes() {
        let mut limits = wgpu::Limits::default();
        for (width, height) in [(0, 1), (1, 0), (limits.max_texture_dimension_2d + 1, 1)] {
            assert!(Rows::new(PixelSize { width, height }, &limits).is_err());
        }
        limits.max_buffer_size = 511;
        assert!(Rows::new(
            PixelSize {
                width: 65,
                height: 1
            },
            &limits
        )
        .is_err());
        limits.max_buffer_size = 512;
        assert!(Rows::new(
            PixelSize {
                width: 65,
                height: 1
            },
            &limits
        )
        .is_ok());
        limits.max_texture_dimension_2d = u32::MAX;
        assert!(Rows::new(
            PixelSize {
                width: u32::MAX,
                height: 1
            },
            &limits
        )
        .is_err());
        assert!(Rows::new(
            PixelSize {
                width: u32::MAX / 4,
                height: 1
            },
            &limits
        )
        .is_err());
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

fn as_bytes<const N: usize>(values: &[[f32; N]]) -> &[u8] {
    // Restrict this cast to initialized f32 arrays: no padding or invalid bytes.
    unsafe { std::slice::from_raw_parts(values.as_ptr().cast(), std::mem::size_of_val(values)) }
}

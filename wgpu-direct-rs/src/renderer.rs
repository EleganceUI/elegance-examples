// The cube resolves directly into a bridge-owned shared GPU image.

use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

use elegance::{Error, ErrorKind, ExternalGpuImage};
use elegance_wgpu::{
    wgpu, Bridge, Config, Error as BridgeError, GpuImageOrigin, RenderPassOptions,
};
use pollster::block_on;
use wgpu::util::DeviceExt;

const SAMPLES: u32 = 4;
const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth24Plus;
const COLOR_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Bgra8Unorm;

pub enum Input {
    Speed(f32),
    Grab(bool),
    Drag { dx: f32, dy: f32 },
    Zoom(f32),
}

pub struct CubeRenderer {
    pub image: ExternalGpuImage,
    pub input: mpsc::Sender<Input>,
}

impl CubeRenderer {
    pub fn new(size: u32, speed: f32) -> Result<Self, Error> {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle());
        let adapter = block_on(instance.request_adapter(&Default::default()))
            .map_err(|error| Error::new(ErrorKind::Gpu, error.to_string()))?;
        let bridge = block_on(Bridge::new(
            &adapter,
            Config {
                width: size,
                height: size,
                origin: GpuImageOrigin::TopLeft,
                format: COLOR_FORMAT,
                max_frames: 4,
            },
        ))
        .map_err(|error| Error::new(ErrorKind::Gpu, error.to_string()))?;

        let image = bridge.image().clone();
        let (input, inputs) = mpsc::channel();
        thread::spawn(move || {
            if let Err(error) = run(bridge, size, speed, inputs) {
                eprintln!("wgpu-direct: {error}");
            }
        });

        Ok(Self { image, input })
    }
}

struct Pose {
    yaw: f32,
    pitch: f32,
    distance: f32,
    speed: f32,
    held: bool,
}

impl Pose {
    fn apply(&mut self, input: Input) {
        match input {
            Input::Speed(speed) => self.speed = speed,
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
    mut bridge: Bridge,
    size: u32,
    speed: f32,
    inputs: mpsc::Receiver<Input>,
) -> Result<(), String> {
    let gpu = Gpu::new(bridge.device().clone(), size, size);
    let mut pose = Pose {
        yaw: 0.0,
        pitch: 0.0,
        distance: 6.0,
        speed,
        held: false,
    };
    let mut last = Instant::now();
    let mut progress = last;

    let result = 'render: loop {
        loop {
            match inputs.try_recv() {
                Ok(input) => pose.apply(input),
                Err(mpsc::TryRecvError::Empty) => break,
                // The mounted UI owns the senders; dropping it stops the producer.
                Err(mpsc::TryRecvError::Disconnected) => break 'render Ok(()),
            }
        }

        let now = Instant::now();
        if now.duration_since(progress) >= Duration::from_secs(10) {
            break Err("no GPU frame could be published for 10 seconds".to_string());
        }
        if !pose.held {
            let turn = pose.speed * (now - last).as_secs_f32();
            pose.yaw += turn;
            pose.pitch += turn * 0.7;
        }

        last = now;
        match gpu.draw(&mut bridge, &pose) {
            Ok(()) => progress = Instant::now(),
            // Backpressure or the UI renderer's device is not ready yet.
            Err(BridgeError::Busy | BridgeError::Publish(2)) => {}
            Err(error) => break Err(format!("cube publication failed: {error}")),
        }

        thread::sleep(
            (now + Duration::from_micros(16_667)).saturating_duration_since(Instant::now()),
        );
    };
    result
}

struct Gpu {
    pipeline: wgpu::RenderPipeline,
    vertices: wgpu::Buffer,
    uniforms: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    samples: wgpu::TextureView,
    depth: wgpu::TextureView,
}

impl Gpu {
    fn new(device: wgpu::Device, width: u32, height: u32) -> Self {
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

        Self {
            pipeline,
            vertices,
            uniforms,
            bind_group,
            samples,
            depth,
        }
    }

    fn draw(&self, bridge: &mut Bridge, pose: &Pose) -> Result<(), BridgeError> {
        bridge.render(|frame| {
            let size = frame.size();
            let transform = mul(
                perspective(size.width as f32 / size.height as f32),
                mul(
                    translation(0.0, 0.0, -pose.distance),
                    mul(rotation_x(pose.pitch), rotation_y(pose.yaw)),
                ),
            );
            frame.write_buffer(&self.uniforms, 0, as_bytes(&[transform]));
            frame.render_pass(
                RenderPassOptions {
                    label: Some("cube"),
                    clear: wgpu::Color {
                        r: 0.07,
                        g: 0.08,
                        b: 0.11,
                        a: 1.0,
                    },
                    multisample: Some(&self.samples),
                    depth_stencil: Some(wgpu::RenderPassDepthStencilAttachment {
                        view: &self.depth,
                        depth_ops: Some(wgpu::Operations {
                            load: wgpu::LoadOp::Clear(1.0),
                            store: wgpu::StoreOp::Discard,
                        }),
                        stencil_ops: None,
                    }),
                },
                |pass| {
                    pass.set_pipeline(&self.pipeline);
                    pass.set_bind_group(0, &self.bind_group, &[]);
                    pass.set_vertex_buffer(0, self.vertices.slice(..));
                    pass.draw(0..36, 0..1);
                },
            );
        })
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

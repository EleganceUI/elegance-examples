// The cube, drawn by wgpu on Direct3D 12 on its own thread, straight into a GpuImage's buffers.
//
// The image shares each buffer as a Direct3D 11 texture and has one shared fence. Direct3D 12
// opens both; wgpu wraps the textures (create_texture_from_hal) and signals the fence after each
// frame's commands (add_signal_fence), so Elegance's GPU waits for the drawing and nothing here
// waits for the GPU.

use std::future::Future;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll, Waker};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use elegance::{GpuFrame, GpuImage, GpuImageSync};
use wgpu::hal::api::Dx12;
use wgpu::util::DeviceExt;
use windows::Win32::Foundation::HANDLE;
use windows::Win32::Graphics::Direct3D12::{ID3D12Fence, ID3D12Resource};

const SAMPLES: u32 = 4;
const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth24Plus;

/// Draws the cube into `image` until stopped or dropped.
pub struct CubeRenderer {
    speed: Arc<AtomicU32>, // degrees per second, as f32 bits
    running: Arc<AtomicBool>,
    thread: Mutex<Option<JoinHandle<()>>>,
}

impl CubeRenderer {
    pub fn new(image: GpuImage, speed: f32) -> Self {
        let speed = Arc::new(AtomicU32::new(speed.to_bits()));
        let running = Arc::new(AtomicBool::new(true));
        let thread = std::thread::spawn({
            let (speed, running) = (speed.clone(), running.clone());
            move || {
                if let Err(message) = run(&image, &speed, &running) {
                    eprintln!("webgpu: {message}");
                }
            }
        });
        Self {
            speed,
            running,
            thread: Mutex::new(Some(thread)),
        }
    }

    /// Degrees per second.
    pub fn set_speed(&self, speed: f32) {
        self.speed.store(speed.to_bits(), Ordering::Relaxed);
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

// One of the image's buffers, imported.
struct Target {
    view: wgpu::TextureView,
    texture: wgpu::Texture,
}

fn run(image: &GpuImage, speed: &AtomicU32, running: &AtomicBool) -> Result<(), String> {
    // Nothing until a window has drawn: the image is made on the renderer's GPU, and the first
    // frame says which one.
    let first = loop {
        if !running.load(Ordering::Relaxed) {
            return Ok(());
        }
        match image.begin_frame() {
            Some(frame) => break frame,
            None => std::thread::sleep(Duration::from_millis(10)),
        }
    };
    let gpu = Gpu::new(&first)?;
    let fence = gpu.open_fence(&first)?;
    let mut targets: Vec<Option<Target>> = (0..GpuImage::BUFFERS).map(|_| None).collect();

    let mut angle = 0.0f32;
    let mut last = Instant::now();
    let mut next = Some(first);
    while running.load(Ordering::Relaxed) {
        let now = Instant::now();
        angle += f32::from_bits(speed.load(Ordering::Relaxed)) * (now - last).as_secs_f32();
        last = now;

        let Some(frame) = next.take().or_else(|| image.begin_frame()) else {
            std::thread::sleep(Duration::from_millis(10));
            continue;
        };
        let target = match &mut targets[frame.buffer as usize] {
            Some(target) => target,
            slot => slot.insert(gpu.import(&frame)?),
        };
        gpu.draw(target, angle, &fence, frame.fence_value);
        image.end_frame(&frame, GpuImageSync::Fence);

        std::thread::sleep((now + Duration::from_micros(16_667)).saturating_duration_since(Instant::now()));
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
    // A device on the GPU Elegance draws with (frame.adapter_luid), which can open its textures.
    fn new(frame: &GpuFrame) -> Result<Self, String> {
        let mut descriptor = wgpu::InstanceDescriptor::new_without_display_handle();
        descriptor.backends = wgpu::Backends::DX12;
        // No debug layer: turning it on removes the D3D12 devices the process already has
        // (Elegance's).
        descriptor.flags = wgpu::InstanceFlags::empty();
        let instance = wgpu::Instance::new(descriptor);
        let adapter = block_on(instance.enumerate_adapters(wgpu::Backends::DX12))
            .into_iter()
            .find(|adapter| {
                let Some(hal) = (unsafe { adapter.as_hal::<Dx12>() }) else {
                    return false;
                };
                let Ok(desc) = (unsafe { hal.as_raw().GetDesc1() }) else {
                    return false;
                };
                let mut luid = [0u8; 8];
                luid[..4].copy_from_slice(&desc.AdapterLuid.LowPart.to_le_bytes());
                luid[4..].copy_from_slice(&desc.AdapterLuid.HighPart.to_le_bytes());
                luid == frame.adapter_luid
            })
            .ok_or("Elegance's GPU has no Direct3D 12 adapter")?;
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
            layout: Some(&device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("cube"),
                bind_group_layouts: &[Some(&layout)],
                immediate_size: 0,
            })),
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
                targets: &[Some(wgpu::TextureFormat::Rgba8Unorm.into())],
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
                        width: frame.width,
                        height: frame.height,
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
        let samples = attachment(wgpu::TextureFormat::Rgba8Unorm);
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

    // The image's fence, the same for every frame.
    fn open_fence(&self, frame: &GpuFrame) -> Result<ID3D12Fence, String> {
        let device = unsafe { self.device.as_hal::<Dx12>() }.ok_or("not a Direct3D 12 device")?;
        let mut fence: Option<ID3D12Fence> = None;
        unsafe { device.raw_device().OpenSharedHandle(HANDLE(frame.fence), &mut fence) }
            .map_err(|error| format!("can't open the image's fence: {error}"))?;
        fence.ok_or_else(|| "can't open the image's fence".into())
    }

    // A buffer's texture, as a wgpu texture to render to. Imported once per buffer.
    fn import(&self, frame: &GpuFrame) -> Result<Target, String> {
        let device = unsafe { self.device.as_hal::<Dx12>() }.ok_or("not a Direct3D 12 device")?;
        let mut resource: Option<ID3D12Resource> = None;
        unsafe {
            device
                .raw_device()
                .OpenSharedHandle(HANDLE(frame.texture), &mut resource)
        }
        .map_err(|error| format!("can't open the image's texture: {error}"))?;
        let resource = resource.ok_or("can't open the image's texture")?;
        drop(device);

        let size = wgpu::Extent3d {
            width: frame.width,
            height: frame.height,
            depth_or_array_layers: 1,
        };
        let format = wgpu::TextureFormat::Rgba8Unorm; // DXGI_FORMAT_R8G8B8A8_UNORM
        let texture = unsafe {
            let raw =
                wgpu::hal::dx12::Device::texture_from_raw(resource, format, wgpu::TextureDimension::D2, size, 1, 1);
            self.device.create_texture_from_hal::<Dx12>(
                raw,
                &wgpu::TextureDescriptor {
                    label: Some("image buffer"),
                    size,
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    format,
                    usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                    view_formats: &[],
                },
                // Shared textures start, and must be left, in the common state.
                wgpu::TextureUses::PRESENT,
            )
        };
        Ok(Target {
            view: texture.create_view(&Default::default()),
            texture,
        })
    }

    fn draw(&self, target: &Target, angle: f32, fence: &ID3D12Fence, fence_value: u64) {
        let aspect = target.texture.width() as f32 / target.texture.height() as f32;
        let transform = mul(
            perspective(aspect),
            mul(
                translation(0.0, 0.0, -6.0),
                mul(rotation_x(angle * 0.7), rotation_y(angle)),
            ),
        );
        self.queue.write_buffer(&self.uniforms, 0, as_bytes(&transform));

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
        // Back to the common state, for Elegance's GPU to read.
        encoder.transition_resources(
            std::iter::empty(),
            std::iter::once(wgpu::TextureTransition {
                texture: &target.texture,
                selector: None,
                state: wgpu::TextureUses::PRESENT,
            }),
        );

        // The fence reaches fence_value once the GPU has run this submission.
        if let Some(queue) = unsafe { self.queue.as_hal::<Dx12>() } {
            queue.add_signal_fence(fence.clone(), fence_value);
        }
        self.queue.submit([encoder.finish()]);
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
            vertices.push([x, y, z, color[0] * shade, color[1] * shade, color[2] * shade]);
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
    [1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, x, y, z, 1.0]
}

fn rotation_x(degrees: f32) -> Mat4 {
    let (s, c) = degrees.to_radians().sin_cos();
    [1.0, 0.0, 0.0, 0.0, 0.0, c, s, 0.0, 0.0, -s, c, 0.0, 0.0, 0.0, 0.0, 1.0]
}

fn rotation_y(degrees: f32) -> Mat4 {
    let (s, c) = degrees.to_radians().sin_cos();
    [c, 0.0, -s, 0.0, 0.0, 1.0, 0.0, 0.0, s, 0.0, c, 0.0, 0.0, 0.0, 0.0, 1.0]
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

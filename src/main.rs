#[cfg(target_os = "android")]
#[macro_use] extern crate android_glue;

mod art;
extern crate winit;
extern crate rand;
extern crate getopts;

#[macro_use]
extern crate vulkano;
extern crate vulkano_win;

use vulkano_win::VkSurfaceBuild;

use std::io::Write;
use getopts::Options;
use std::env;

use std::time::{Duration, Instant};

use rand::Rng;

use std::sync::Arc;

use art::{aa_fs, compose_fs, draw_vs};

use std::f64;
use std::i16;

//#[cfg(target_os = "android")]
//android_start!(main);

mod renderpass {
    single_pass_renderpass!{
    attachments: {
      colour: {
        load: Clear,
        store: Store,
        format: ::vulkano::format::A1R5G5B5UnormPack16,
      }
    },
    pass: {
      color: [colour],
      depth_stencil: {}
    }
  }
}

/// # Here
///
/// * `What` - What is this do it do?
/// * Gub gub
///
/// Here is a title, Maybe this?
/// Creates a swapchain and framebuffers
fn create_swapchain(device: &Arc<vulkano::device::Device>,
                    surface: &Arc<vulkano::swapchain::Surface>,
                    caps: &vulkano::swapchain::Capabilities,
                    renderpass: &Arc<renderpass::CustomRenderPass>,
                    sharing: vulkano::sync::SharingMode,
                    dimensions: [u32; 2],
                    old_swapchain: Option<&Arc<vulkano::swapchain::Swapchain>>)
                    -> (Arc<vulkano::swapchain::Swapchain>,
                        Vec<Arc<vulkano::framebuffer::Framebuffer<renderpass::CustomRenderPass>>>) {
    let (swapchain, images) = {
        use vulkano::swapchain::{Swapchain, SurfaceTransform, CompositeAlpha};
        use vulkano::format::B8G8R8A8Unorm;

        let present = caps.present_modes.iter().next().unwrap();
        let usage = caps.supported_usage_flags;

        Swapchain::new(&device,
                       surface,
                       2,
                       B8G8R8A8Unorm,
                       dimensions,
                       1,
                       &usage,
                       sharing,
                       SurfaceTransform::Identity,
                       CompositeAlpha::Opaque,
                       present,
                       true,
                       old_swapchain)
            .expect("failed to create swapchain")
    };

    let framebuffers = images.iter()
        .map(|image| {
            let attachments = renderpass::AList { colour: &image };

            vulkano::framebuffer::Framebuffer::new(&renderpass,
                                                   [images[0].dimensions()[0],
                                                    images[0].dimensions()[1],
                                                    1],
                                                   attachments)
                .unwrap()
        })
        .collect::<Vec<_>>();

    (swapchain, framebuffers)
}

fn build_window(instance: &Arc<vulkano::instance::Instance>, fullscreen: bool, resolution: [u32; 2]) -> vulkano_win::Window {
  let mut window = winit::WindowBuilder::new()
    .with_title("Abstract Art".to_string())
    .with_dimensions(resolution[0], resolution[1]);
  
  if fullscreen {
    window = window.with_fullscreen(winit::get_primary_monitor())
  }
  
  window
    .build_vk_surface(&instance)
    .unwrap()
}

fn main() {

    let args: Vec<String> = env::args().collect();
    let program = args[0].clone();

    let mut opts = Options::new();
    opts.optopt("g", "battle-group", "Battle Group by index", "ID");
    opts.optopt("s", "screen-size", "Size of the screen", "WIDTHxHEIGHT");
    opts.optopt("f", "fps", "Framesrate", "FPS");
    opts.optflag("h", "help", "print this help menu");
    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(f) => panic!(f.to_string()),
    };

    if matches.opt_present("h") {
        let brief = format!("Usage: {} [options]", program);
        print!("{}", opts.usage(&brief));
        return;
    }

    let mut image_index = match matches.opt_str("g") {
        Some(id) => {
            match id.parse::<u16>() {
                Ok(id) => id,
                Err(_) => {
                    writeln!(&mut std::io::stderr(),
                             "Battle group id must be a positive integer.")
                        .expect("Failed to print error?!?!");
                    rand::thread_rng().gen_range(0, art::BATTLE_GROUP_MAX)
                }
            }
        }
        None => rand::thread_rng().gen_range(0, art::BATTLE_GROUP_MAX),
    };

    let fps = match matches.opt_str("f") {
      Some(fps) => {
        match fps.parse::<f32>() {
          Ok(fps) => fps,
          Err(_) => {
            writeln!(&mut std::io::stderr(),
                     "Failed to parse framerate")
                .expect("Failed to print error?!?!");
            60.
          }
        }
      },
      None => 60.
    };

    let mut dimensions = [art::MAP_WIDTH, art::MAP_HEIGHT];
    let mut resolution = dimensions;
    match matches.opt_str("s") {
      Some(res) => {
        for (i, v) in res.split("x").take(2).enumerate() {
          match v.parse::<u32>() {
            Ok(v) => {resolution[i] = v;},
            Err(_) => {
              writeln!(&mut std::io::stderr(),
                       "Screen size must be positive intager.")
                  .expect("Failed to print error?!?!");
              break;
            }
          };
        }
      },
      None => (),
    };


    let extensions = vulkano::instance::InstanceExtensions {
        ext_debug_report: true,
        ..vulkano_win::required_extensions()
    };
    
    println!("List of Vulkan debugging layers available to use:");
    let mut layers = vulkano::instance::layers_list().unwrap();
    while let Some(l) = layers.next() {
        println!("\t{}", l.name());
    }

    let layers = vec![
      // "VK_LAYER_LUNARG_threading",
      // "VK_LAYER_LUNARG_swapchain",
      // "VK_LAYER_LUNARG_image",
      // "VK_LAYER_LUNARG_param_checker",
      // "VK_LAYER_GOOGLE_unique_objects",
      // "VK_LAYER_LUNARG_draw_state",
      // "VK_LAYER_LUNARG_device_limits",
      // "VK_LAYER_LUNARG_mem_tracker",
      // "VK_LAYER_LUNARG_object_tracker",
    ];
    
//    let extensions = ;
    let instance = vulkano::instance::Instance::new(None, &extensions, &layers)
        .expect("failed to create instance");

    let all = vulkano::instance::debug::MessageTypes {
        error: true,
        warning: true,
        performance_warning: true,
        information: false,
        debug: true,
    };

    let _debug_callback = vulkano::instance::debug::DebugCallback::new(&instance, all, |msg| {
        let ty = if msg.ty.error {
            "error"
        } else if msg.ty.warning {
            "warning"
        } else if msg.ty.performance_warning {
            "performance_warning"
        } else if msg.ty.information {
            "information"
        } else if msg.ty.debug {
            "debug"
        } else {
            panic!("no-impl");
        };
        println!("{} {}: {}", msg.layer_prefix, ty, msg.description);
    }).ok();

    let physical = vulkano::instance::PhysicalDevice::enumerate(&instance)
        .next()
        .expect("no device available");
    println!("Using device: {} (type: {:?})",
             physical.name(),
             physical.ty());

    let mut fullscreen = false;
    let mut window = build_window(&instance, fullscreen, dimensions);

    let queue = physical.queue_families()
        .find(|q| q.supports_graphics() && window.surface().is_supported(q).unwrap_or(false))
        .expect("couldn't find a graphical queue family");

    let device_ext = vulkano::device::DeviceExtensions {
        khr_swapchain: true,
        ..vulkano::device::DeviceExtensions::none()
    };
    let (device, mut queues) = vulkano::device::Device::new(&physical,
                                                            physical.supported_features(),
                                                            &device_ext,
                                                            [(queue, 0.5)].iter().cloned())
        .expect("failed to create device");
    let queue = queues.next().unwrap();

    let caps = window.surface()
        .get_capabilities(&physical)
        .expect("failed to get surface capabilities");
    //dimensions = caps.current_extent.unwrap_or(dimensions);

    let renderpass = renderpass::CustomRenderPass::new(&device,
                                                       &renderpass::Formats {
                                                           colour: (vulkano::format::A1R5G5B5UnormPack16,
                                                                   1),
                                                       })
        .unwrap();

    let (mut swapchain, mut framebuffers) =
        create_swapchain(&device,
                         &window.surface(),
                         &caps,
                         &renderpass,
                         vulkano::sync::SharingMode::from(&queue),
                         dimensions,
                         None);

    let vertex_buffer = {
        use vulkano::buffer::cpu_access::CpuAccessibleBuffer;
        use vulkano::buffer::BufferUsage;

        #[derive(Clone)]
        struct Vertex {
            position: [f32; 2],
        }
        impl_vertex!(Vertex, position);

        CpuAccessibleBuffer::from_iter(&device,
                                       &BufferUsage::vertex_buffer(),
                                       Some(queue.family()),
                                       [Vertex { position: [0., 0.] },
                                        Vertex { position: [4., 0.] },
                                        Vertex { position: [0., 4.] }]
                                           .iter()
                                           .cloned())
            .expect("failed to create buffer")
    };

    let start_time = Instant::now();
    let get_time = || {
        let t = Instant::now().duration_since(start_time);
        t.as_secs() as f64 + t.subsec_nanos() as f64 / 1000000000.
    };

    struct LayerBufferSet {
      map: Arc<vulkano::buffer::cpu_access::CpuAccessibleBuffer<[[u8; 1]; (art::MAP_WIDTH * art::MAP_HEIGHT) as usize]>>,
      palette: Arc<vulkano::buffer::cpu_access::CpuAccessibleBuffer<[[u16; 1]; 8 * art::PALETTE_MAX as usize]>>,
      palette_cycles: Arc<vulkano::buffer::cpu_access::CpuAccessibleBuffer<aa_fs::ty::PaletteCycles>>,
      translations: Arc<vulkano::buffer::cpu_access::CpuAccessibleBuffer<aa_fs::ty::Translations>>,
      distortions: Arc<vulkano::buffer::cpu_access::CpuAccessibleBuffer<aa_fs::ty::Distortions>>
    };
    impl LayerBufferSet {
        fn new(device: &Arc<vulkano::device::Device>,
               queue_family: vulkano::instance::QueueFamily)
               -> LayerBufferSet {
            use art::aa_fs::ty::{PaletteCycles, Translations, Distortions};
            use vulkano::buffer::cpu_access::CpuAccessibleBuffer;
            use vulkano::buffer::BufferUsage;

            LayerBufferSet {
          map: CpuAccessibleBuffer::from_data(&device,
                                              &BufferUsage::uniform_buffer(),
                                              Some(queue_family),
                                              [[0; 1]; (art::MAP_WIDTH * art::MAP_HEIGHT) as usize])
               .expect("failed to create buffer"),
          palette: CpuAccessibleBuffer::from_data(&device,
                                                  &BufferUsage::uniform_buffer(),
                                                  Some(queue_family),
                                                  [[0; 1]; 8 * art::PALETTE_MAX as usize])
                   .expect("failed to create palette buffer"),
            palette_cycles: CpuAccessibleBuffer::<PaletteCycles>::from_data(&device,
                                                         &BufferUsage::uniform_buffer(),
                                                         Some(queue_family),
                                                         Default::default())
                                                         .expect("failed to create buffer"),
            translations: CpuAccessibleBuffer::<Translations>::from_data(&device,
                                                         &BufferUsage::uniform_buffer(),
                                                         Some(queue_family),
                                                         Default::default())
                                                         .expect("failed to create buffer"),
            distortions: CpuAccessibleBuffer::<Distortions>::from_data(&device,
                                                         &BufferUsage::uniform_buffer(),
                                                         Some(queue_family),
                                                         Default::default())
                                                         .expect("failed to create buffer")
        }
        }
    };

    let global_buffer = vulkano::buffer::cpu_access::CpuAccessibleBuffer::from_data(&device,
                                        &vulkano::buffer::BufferUsage::uniform_buffer(),
                                        Some(queue.family()),
                                        aa_fs::ty::Globals {
                                            time: get_time(),
                                            screen_size: dimensions,
                                            fps: fps,
                                        })
            .expect("failed to create buffer");

    let layer_buffers = [LayerBufferSet::new(&device, queue.family()),
                         LayerBufferSet::new(&device, queue.family())];

    let draw_vs = draw_vs::Shader::load(&device).expect("failed to create shader module");
    let aa_fs = aa_fs::Shader::load(&device).expect("failed to create shader module");
    let compose_fs = compose_fs::Shader::load(&device).expect("failed to create shader module");

    let get_image = |id| {
        use art::battle_group::BattleGroup;
        println!("Battle Group: {}", id);
        let bg = match BattleGroup::for_index(id) {
            Ok(v) => v,
            Err(e) => {
                println!("Error: {}", e);
                Default::default()
            }
        };

        for (buffers, layer) in layer_buffers.iter().zip(bg.layers.iter()) {
            // Write map
            {
                let mut mapping = buffers.map.write(Duration::new(0, 0)).unwrap();
                for (o, i) in mapping.iter_mut().zip(layer.map.iter()) {
                    o[0] = *i;
                }
            }
            // Write palette
            {
                let mut mapping = buffers.palette.write(Duration::new(0, 0)).unwrap();
                for (o, i) in mapping.iter_mut().zip(layer.palettes[0].iter()) {
                    o[0] = *i;
                }
                let mut o = mapping.iter_mut();
                for p in layer.palettes.iter() {
                    for c in p.iter() {
                        o.next().unwrap()[0] = *c;
                    }
                }
            }
            // Write palette cycle
            {
                let mut mapping = buffers.palette_cycles.write(Duration::new(0, 0)).unwrap();
                // This is frameskip?
                // TODO Figure out why multiplyier
                mapping.speed = 7.55 * (1 + layer.speed) as f32;//(1. / ((1 + layer.speed) as f64 * 6.)) as f32;
                mapping.style = layer.style.clone() as u32;
                mapping.cycles[0].start = layer.cycles[0].start as f32 / 16.;
                mapping.cycles[0].end = layer.cycles[0].end as f32 / 16.;
                // println!("Cycle:( speed: {}, cycles: {:?} )", layer.speed, layer.cycles);
            }
            // Write translations
            {
                let mut mapping = buffers.translations.write(Duration::new(0, 0)).unwrap();
                let c1 = fps / art::MAP_WIDTH as f32;
                for (l, m) in layer.translations.iter().zip(mapping.translations.iter_mut()) {
                    match l {
                        &Some(ref l) => {
                            m.duration = (l.duration as f32) / fps;
                            m.velocity = [
                              (l.velocity.0 as f32 * c1),
                              (l.velocity.1 as f32 * c1)
                            ];
                            m.acceleration = [
                              (l.acceleration.0 as f32 * c1),
                              (l.acceleration.1 as f32 * c1)
                            ];
                            // println!("{:?}", l);
                        }
                        &None => {
                            m.duration = 0.;
                            m.velocity = [0.; 2];
                            m.acceleration = [0.; 2];
                        }
                    };
                }
            }
            {
                const C1: f64 = 1. / 512.;
                const C2: f64 = f64::consts::PI / i16::MAX as f64;
                const C3: f64 = f64::consts::PI / 60.;
                let mut mapping = buffers.distortions.write(Duration::new(0, 0)).unwrap();
                for (l, m) in layer.distortions.iter().zip(mapping.distortions.iter_mut()) {
                    match l {
                        &Some(ref l) => {
                            m.duration = (l.duration as f32) / fps;
                            m.style = l.style.clone() as u32;
                            m.amplitude = (C1 * l.amplitude as f64) as f32;
                            m.amplitude_delta = (2. * C1 * l.amplitude_delta as f64) as f32;
                            m.frequency = (C2 * l.frequency as f64) as f32;
                            m.frequency_delta = (2. * C2 * l.frequency_delta as f64) as f32;
                            m.compression = l.compression as f32;
                            m.compression_delta = l.compression_delta as f32;
                            m.speed = (C3 * (l.speed as f64)) as f32;
                            // println!("{:?}", l);
                        }
                        &None => {
                            m.duration = 0.;
                            m.style = 0;
                            m.frequency = 0.;
                            m.amplitude = 0.;
                            m.compression = 0.;
                            m.speed = 0.;
                        }
                    }
                }
            }
        }
    };

    get_image(image_index);

    let mut change_image = |i| {
        let ni = i + image_index as i16;
        if ni < 0 {
            image_index = art::BATTLE_GROUP_MAX - 1;
        } else if ni >= art::BATTLE_GROUP_MAX as i16 {
            image_index = 0;
        } else {
            image_index = ni as u16;
        }
        get_image(image_index);
    };

    // TODO: Rename all these
    // TODO: Consider alternative format?
    let gen_tex = || vulkano::image::attachment::AttachmentImage::new(&device,
                                                       [art::MAP_WIDTH,
                                                        art::MAP_HEIGHT],
                                                        vulkano::format::A1R5G5B5UnormPack16).unwrap();
    let art_tex = [
      gen_tex(),
      gen_tex(),
    ];

    //    let layer_framebuffers = layers.iter().map(|layer| {
    // use vulkano::framebuffer::Framebuffer;
    // Framebuffer::new(&renderpass,
    // [art::MAP_WIDTH, art::MAP_HEIGHT, 1],
    // renderpass::AList { color: &art_tex }).unwrap()
    // }).collect::<Vec<_>>();

    // TODO: do this good
    let layer_framebuffers =
        [vulkano::framebuffer::Framebuffer::new(&renderpass,
                                                [art::MAP_WIDTH, art::MAP_HEIGHT, 1],
                                                renderpass::AList { colour: &art_tex[0] })
             .unwrap(),
         vulkano::framebuffer::Framebuffer::new(&renderpass,
                                                [art::MAP_WIDTH, art::MAP_HEIGHT, 1],
                                                renderpass::AList { colour: &art_tex[1] })
             .unwrap()];

    let (map_texture, palette_texture) = {
        use vulkano::image::immutable::ImmutableImage;
        use vulkano::image::Dimensions;
        use vulkano::format::{R8Unorm, B5G5R5A1UnormPack16};
        (ImmutableImage::new(&device,
                             Dimensions::Dim2d {
                                 width: art::MAP_WIDTH,
                                 height: art::MAP_HEIGHT,
                             },
                             R8Unorm,
                             Some(queue.family()))
            .unwrap(),
         ImmutableImage::new(&device,
                             Dimensions::Dim2d {
                                 width: art::PALETTE_MAX as u32,
                                 height: 8,
                             },
                             B5G5R5A1UnormPack16,
                             Some(queue.family()))
            .unwrap())
    };

    let (map_sampler, palette_sampler) = {
        use vulkano::sampler::{Sampler, Filter, MipmapMode, SamplerAddressMode};
        (Sampler::new(&device,
                      Filter::Nearest,
                      Filter::Nearest,
                      MipmapMode::Nearest,
                      SamplerAddressMode::Repeat,
                      SamplerAddressMode::Repeat,
                      SamplerAddressMode::Repeat,
                      0.0,
                      1.0,
                      0.0,
                      0.0)
            .unwrap(),
         Sampler::new(&device,
                      Filter::Nearest,
                      Filter::Nearest,
                      MipmapMode::Nearest,
                      SamplerAddressMode::ClampToEdge,
                      SamplerAddressMode::ClampToEdge,
                      SamplerAddressMode::Repeat,
                      0.0,
                      1.0,
                      0.0,
                      0.0)
            .unwrap())
    };

    let art_descriptor_pool = vulkano::descriptor::descriptor_set::DescriptorPool::new(&device);
    mod art_pipeline_layout {
        pipeline_layout!{
            set0: {
                map: CombinedImageSampler,
                palette: CombinedImageSampler,
                global: UniformBuffer<::art::aa_fs::ty::Globals>,
                pc: UniformBuffer<::art::aa_fs::ty::PaletteCycles>,
                translations: UniformBuffer<::art::aa_fs::ty::Translations>,
                distortions: UniformBuffer<::art::aa_fs::ty::Distortions>
            }
        }
    }
    let art_pipeline_layout = art_pipeline_layout::CustomPipeline::new(&device).unwrap();

    let layer_sets = layer_buffers.iter()
        .map(|buffers| {
            art_pipeline_layout::set0::Set::new(&art_descriptor_pool,
                                                &art_pipeline_layout,
                                                &art_pipeline_layout::set0::Descriptors {
                                                    map: (&map_sampler, &map_texture),
                                                    palette: (&palette_sampler, &palette_texture),
                                                    global: &global_buffer,
                                                    pc: &buffers.palette_cycles,
                                                    translations: &buffers.translations,
                                                    distortions: &buffers.distortions,
                                                })
        })
        .collect::<Vec<_>>();


    let descriptor_pool = vulkano::descriptor::descriptor_set::DescriptorPool::new(&device);
    mod pipeline_layout {
        pipeline_layout!{
      set0: {
        bg3: CombinedImageSampler,
        bg4: CombinedImageSampler
      }
    }
    }

    let pipeline_layout = pipeline_layout::CustomPipeline::new(&device).unwrap();
    let set = pipeline_layout::set0::Set::new(&descriptor_pool,
                                              &pipeline_layout,
                                              &pipeline_layout::set0::Descriptors {
                                                  bg3: (&map_sampler, &art_tex[0]),
                                                  bg4: (&map_sampler, &art_tex[1]),
                                              });

    let art_pipeline = {
        let dim = map_texture.dimensions().width_height();
        vulkano::pipeline::GraphicsPipeline::new(&device,
      vulkano::pipeline::GraphicsPipelineParams {
        vertex_input: vulkano::pipeline::vertex::SingleBufferDefinition::new(),
        vertex_shader: draw_vs.main_entry_point(),
        input_assembly: vulkano::pipeline::input_assembly::InputAssembly {
            topology: vulkano::pipeline::input_assembly::PrimitiveTopology::TriangleStrip,
            primitive_restart_enable: false,
        },
        tessellation: None,
        geometry_shader: None,
        viewport: vulkano::pipeline::viewport::ViewportsState::Fixed {
          data: vec![(
            vulkano::pipeline::viewport::Viewport {
              origin: [0.0, 0.0],
              depth_range: 0.0 .. 1.0,
              dimensions: [dim[0] as f32, dim[1] as f32],
            },
            vulkano::pipeline::viewport::Scissor::irrelevant()
          )]
        },
        raster: Default::default(),
        multisample: vulkano::pipeline::multisample::Multisample::disabled(),
        fragment_shader: aa_fs.main_entry_point(),
        depth_stencil: vulkano::pipeline::depth_stencil::DepthStencil::disabled(),
        blend: vulkano::pipeline::blend::Blend::pass_through(),
        layout: &art_pipeline_layout,
        render_pass: vulkano::framebuffer::Subpass::from(&renderpass, 0).unwrap(),
      }
    ).unwrap()
    };

    let pipeline = {
      vulkano::pipeline::GraphicsPipeline::new(&device, vulkano::pipeline::GraphicsPipelineParams {
        vertex_input: vulkano::pipeline::vertex::SingleBufferDefinition::new(),
        vertex_shader: draw_vs.main_entry_point(),
        input_assembly: vulkano::pipeline::input_assembly::InputAssembly {
          topology: vulkano::pipeline::input_assembly::PrimitiveTopology::TriangleStrip,
          primitive_restart_enable: false,
        },
        tessellation: None,
        geometry_shader: None,
        viewport: vulkano::pipeline::viewport::ViewportsState::DynamicViewports {
          scissors: vec![
            vulkano::pipeline::viewport::Scissor::irrelevant()
          ],
        },
        raster: Default::default(),
        multisample: vulkano::pipeline::multisample::Multisample::disabled(),
        fragment_shader: compose_fs.main_entry_point(),
        depth_stencil: vulkano::pipeline::depth_stencil::DepthStencil::disabled(),
        blend: vulkano::pipeline::blend::Blend::pass_through(),
        layout: &pipeline_layout,
        render_pass: vulkano::framebuffer::Subpass::from(&renderpass, 0).unwrap(),
      }).unwrap()
    };

    let mut state = {
        let dim: [f32; 2] = [dimensions[0] as f32, dimensions[1] as f32];
        vulkano::command_buffer::DynamicState {
            viewports: Some(vec![vulkano::pipeline::viewport::Viewport {
                                     origin: [0., 0.],
                                     depth_range: 0. .. 1.,
                                     dimensions: dim,
                                 }]),
            scissors: None,
            line_width: None,
        }
    };

    let build_command_buffers = |state, framebuffers: &Vec<Arc<vulkano::framebuffer::Framebuffer<renderpass::CustomRenderPass>>>| framebuffers.iter().map(|framebuffer| {
        use vulkano::buffer::BufferSlice;
        use vulkano::command_buffer::{PrimaryCommandBufferBuilder, DynamicState};
        let mut command_buffer = PrimaryCommandBufferBuilder::new(&device, queue.family());
        for ((buffers, layer_framebuffer), layer_set) in layer_buffers.iter().zip(layer_framebuffers.iter()).zip(layer_sets.iter()) {
            command_buffer = command_buffer
                .copy_buffer_to_color_image(BufferSlice::from(&buffers.map), &map_texture, 0, 0 .. 1, [0, 0, 0],
                    [map_texture.dimensions().width(), map_texture.dimensions().height(), 1])
                .copy_buffer_to_color_image(BufferSlice::from(&buffers.palette),
                    &palette_texture, 0, 0 .. 1, [0, 0, 0],
                    [palette_texture.dimensions().width(), palette_texture.dimensions().height(), 1])
                // Layer 1
                .draw_inline(&renderpass, &layer_framebuffer, renderpass::ClearValues {
                    colour: [0.0, 0.0, 0.0, 0.0]
                })
                .draw(&art_pipeline, &vertex_buffer,
                    &DynamicState::none(), layer_set, &())
                .draw_end();
        }
        command_buffer
            // Compose
            .draw_inline(&renderpass, &framebuffer, renderpass::ClearValues {
                colour: [0.0, 0.0, 0.0, 1.0]
            })
            .draw(&pipeline, &vertex_buffer, &state, &set, &())
            .draw_end()
            
            .build()
    }).collect::<Vec<_>>();

    let mut command_buffers = build_command_buffers(state, &framebuffers);

    let mut submissions: Vec<Arc<vulkano::command_buffer::Submission>> = Vec::new();

    'run: loop {
        submissions.retain(|s| s.destroying_would_block());
        
        {
            let mut mapping = global_buffer.write(Duration::new(1, 0)).unwrap();
            mapping.time = get_time();
        }

        let image_num = swapchain.acquire_next_image(Duration::from_millis(1)).unwrap();
        submissions.push(vulkano::command_buffer::submit(&command_buffers[image_num], &queue).unwrap());
        swapchain.present(&queue, image_num).unwrap();

        let mut fullscreen_toggle = fullscreen;
        let mut rebuild_swapchain = false;
        let mut use_old_swapchain = false;

        for ev in window.window().poll_events() {
            use winit::{Event, VirtualKeyCode, ElementState, MouseScrollDelta, TouchPhase};
            match ev {
                Event::Closed => break 'run,
                Event::KeyboardInput(_, _, Some(VirtualKeyCode::Escape)) => break 'run,
                Event::KeyboardInput(ElementState::Pressed, _, Some(VirtualKeyCode::F11)) => fullscreen_toggle = !fullscreen_toggle,
                Event::Resized(width, height) => {
                    dimensions = [width, height];
                    rebuild_swapchain = true;
                    use_old_swapchain = true;
                }
                Event::MouseWheel(MouseScrollDelta::LineDelta(_, d), TouchPhase::Moved) => {
                    change_image(d as i16)
                }
                Event::KeyboardInput(ElementState::Pressed, _, Some(VirtualKeyCode::Up)) => {
                    change_image(1)
                }
                Event::KeyboardInput(ElementState::Pressed, _, Some(VirtualKeyCode::Down)) => {
                    change_image(-1)
                }
                _ => (),
            }
        }
        if fullscreen != fullscreen_toggle {
          fullscreen = fullscreen_toggle;
          if !fullscreen {
            dimensions = [art::MAP_WIDTH, art::MAP_HEIGHT];
          }
          window = build_window(&instance, fullscreen, dimensions);
          rebuild_swapchain = true;
          use_old_swapchain = false;
        }

        if rebuild_swapchain {
            let sc = {
              let old_swapchain =
                if use_old_swapchain {
                  Some(&swapchain)
                }
                else {
                  None
                };
              create_swapchain(&device,
                                      &window.surface(),
                                      &caps,
                                      &renderpass,
                                      vulkano::sync::SharingMode::from(&queue),
                                      dimensions,
                                      old_swapchain)
            };
            
            swapchain = sc.0;
            framebuffers = sc.1;
        }

        state = {
            let dim: [f32; 2] = [dimensions[0] as f32, dimensions[1] as f32];
            vulkano::command_buffer::DynamicState {
                viewports: Some(vec![vulkano::pipeline::viewport::Viewport {
                                         origin: [0., 0.],
                                         depth_range: 0. .. 1.,
                                         dimensions: dim,
                                     }]),
                scissors: None,
                line_width: None,
            }
        };
        command_buffers = build_command_buffers(state, &framebuffers);
      
    }
}

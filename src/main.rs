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

//mod renderpass {
//    single_pass_renderpass!{
//    attachments: {
//      colour: {
//        load: Clear,
//        store: Store,
//        format: ::vulkano::format::A1R5G5B5UnormPack16,
//      }
//    },
//    pass: {
//      color: [colour],
//      depth_stencil: {}
//    }
//  }
//}

/// # Here
///
/// * `What` - What is this do it do?
/// * Gub gub
///
/// Here is a title, Maybe this?
/// Creates a swapchain and framebuffers
fn create_swapchain(device: Arc<vulkano::device::Device>,
                    surface: Arc<vulkano::swapchain::Surface>,
                    caps: &vulkano::swapchain::Capabilities,
                    sharing: vulkano::sync::SharingMode,
                    dimensions: [u32; 2],
                    old_swapchain: Option<&Arc<vulkano::swapchain::Swapchain>>)
                    -> (Arc<vulkano::swapchain::Swapchain>,
                        Vec<Arc<vulkano::image::swapchain::SwapchainImage>>) {
    let (swapchain, images) = {
        use vulkano::swapchain::{Swapchain, SurfaceTransform, CompositeAlpha};
        use vulkano::format::B8G8R8A8Unorm;

        let present = caps.present_modes.iter().next().unwrap();
        let usage = caps.supported_usage_flags;

        Swapchain::new(device,
                       surface,
                       caps.min_image_count,
                       B8G8R8A8Unorm,
                       dimensions,
                       1,
                       usage,
                       sharing,
                       SurfaceTransform::Identity,
                       CompositeAlpha::Opaque,
                       present,
                       true,
                       old_swapchain)
            .expect("failed to create swapchain")
    };

    (swapchain, images)
}
fn create_framebuffers<Rp: vulkano::framebuffer::RenderPassAbstract>(renderpass: Arc<Rp>,
                       images: Vec<Arc<vulkano::image::swapchain::SwapchainImage>>
                   ) -> Vec<Arc<vulkano::framebuffer::Framebuffer<Arc<Rp>, ((), std::sync::Arc<vulkano::image::SwapchainImage>)>>> {
    let framebuffers = images.iter()
        .map(|image| {
            Arc::new(vulkano::framebuffer::Framebuffer::start(renderpass)
                .add(image.clone()).unwrap()
                .build().unwrap())
        })
        .collect::<Vec<_>>();
    framebuffers
}

fn build_window(events_loop: &winit::EventsLoop, instance: &Arc<vulkano::instance::Instance>, fullscreen: bool, resolution: [u32; 2]) -> vulkano_win::Window {
  let mut window = winit::WindowBuilder::new()
    .with_title("Abstract Art".to_string())
    .with_dimensions(resolution[0], resolution[1]);
  
  if fullscreen {
    window = window.with_fullscreen(winit::get_primary_monitor())
  }
  let events_loop = winit::EventsLoop::new();
  window
    .build_vk_surface(&events_loop, instance.clone())
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

    let events_loop = winit::EventsLoop::new();
    let mut fullscreen = false;
    let mut window = build_window(&events_loop, &instance, fullscreen, dimensions);

    let queue = physical.queue_families()
        .find(|q| q.supports_graphics() && window.surface().is_supported(*q).unwrap_or(false))
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

    //let renderpass = renderpass::CustomRenderPass::new(&device,
    //                                                   &renderpass::Formats {
    //                                                       colour: (vulkano::format::A1R5G5B5UnormPack16,
    //                                                               1),
    //                                                   })
    //    .unwrap();

    let (mut swapchain, mut images) =
        create_swapchain(device,
                         window.surface().clone(),
                         &caps,
                         vulkano::sync::SharingMode::from(&queue),
                         dimensions,
                         None);
    
    let renderpass = Arc::new(single_pass_renderpass!(device.clone(),
        attachments: {
            colour: {
                load: Clear,
                store: Store,
                format: swapchain.format(),
                samples: 1,
            }
        },
        pass: {
          color: [colour],
          depth_stencil: {}
        }
    ).unwrap());
    
    let mut framebuffers = create_framebuffers(renderpass.clone(), images);

    let vertex_buffer = {
        use vulkano::buffer::cpu_access::CpuAccessibleBuffer;
        use vulkano::buffer::BufferUsage;

        #[derive(Clone)]
        struct Vertex {
            position: [f32; 2],
        }
        impl_vertex!(Vertex, position);

        CpuAccessibleBuffer::from_iter(device.clone(),
                                       BufferUsage::vertex_buffer(),
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
        fn new(device: Arc<vulkano::device::Device>,
               queue_family: vulkano::instance::QueueFamily)
               -> LayerBufferSet {
            use art::aa_fs::ty::{PaletteCycles, PaletteCycle, Translations, Translation, Distortions, Distortion};
            use vulkano::buffer::cpu_access::CpuAccessibleBuffer;
            use vulkano::buffer::BufferUsage;

            LayerBufferSet {
          map: CpuAccessibleBuffer::from_data(device.clone(),
                                              BufferUsage::uniform_buffer(),
                                              Some(queue_family),
                                              [[0; 1]; (art::MAP_WIDTH * art::MAP_HEIGHT) as usize])
               .expect("failed to create buffer"),
          palette: CpuAccessibleBuffer::from_data(device.clone(),
                                                  BufferUsage::uniform_buffer(),
                                                  Some(queue_family),
                                                  [[0; 1]; 8 * art::PALETTE_MAX as usize])
                   .expect("failed to create palette buffer"),
            palette_cycles: CpuAccessibleBuffer::<PaletteCycles>::from_data(device.clone(),
                                                         BufferUsage::uniform_buffer(),
                                                         Some(queue_family),
                                                         PaletteCycles {
                                                             cycles: [PaletteCycle{
                                                                 start: 0.,
                                                                 end: 0.,
                                                                 _dummy0: [0; 8]
                                                             }; 2],
                                                             style: 0,
                                                             speed: 0.,
                                                             _dummy0: [0; 8],
                                                         })
                                                         .expect("failed to create buffer"),
            translations: CpuAccessibleBuffer::<Translations>::from_data(device.clone(),
                                                         BufferUsage::uniform_buffer(),
                                                         Some(queue_family),
                                                        Translations {
                                                            translations: [Translation {
                                                                acceleration: [0.; 2],
                                                                duration: 0.,
                                                                velocity: [0.; 2],
                                                                _dummy0: [0; 4],
                                                                _dummy1: [0; 8]
                                                            }; 4],
                                                        })
                                                         .expect("failed to create buffer"),
            distortions: CpuAccessibleBuffer::<Distortions>::from_data(device.clone(),
                                                         BufferUsage::uniform_buffer(),
                                                         Some(queue_family),
                                                         Distortions {
                                                             distortions: [Distortion {
                                                                 amplitude: 0.,
                                                                 amplitude_delta: 0.,
                                                                 compression: 0.,
                                                                 compression_delta: 0.,
                                                                 duration: 0.,
                                                                 frequency: 0.,
                                                                 frequency_delta: 0.,
                                                                 speed: 0.,
                                                                 style: 0,
                                                                 _dummy0: [0; 12]
                                                             }; 4],
                                                         })
                                                         .expect("failed to create buffer")
        }
        }
    };

    let global_buffer = vulkano::buffer::cpu_access::CpuAccessibleBuffer::from_data(device.clone(),
                                        vulkano::buffer::BufferUsage::uniform_buffer(),
                                        Some(queue.family()),
                                        aa_fs::ty::Globals {
                                            time: get_time(),
                                            screen_size: dimensions,
                                            fps: fps,
                                        })
            .expect("failed to create buffer");

    let layer_buffers = [LayerBufferSet::new(device.clone(), queue.family()),
                         LayerBufferSet::new(device.clone(), queue.family())];

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
                let mut mapping = buffers.map.write().unwrap();
                for (o, i) in mapping.iter_mut().zip(layer.map.iter()) {
                    o[0] = *i;
                }
            }
            // Write palette
            {
                let mut mapping = buffers.palette.write().unwrap();
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
                let mut mapping = buffers.palette_cycles.write().unwrap();
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
                let mut mapping = buffers.translations.write().unwrap();
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
                let mut mapping = buffers.distortions.write().unwrap();
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
    let gen_tex = || vulkano::image::attachment::AttachmentImage::new(device.clone(),
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
        [vulkano::framebuffer::Framebuffer::start(renderpass.clone())
            //.with_dimensions([art::MAP_WIDTH, art::MAP_HEIGHT, 1])
            .add(art_tex[0].clone()).unwrap()
            .build()
            .unwrap(),
         vulkano::framebuffer::Framebuffer::start(renderpass.clone())
            //.with_dimensions([art::MAP_WIDTH, art::MAP_HEIGHT, 1])
            .add(art_tex[1].clone()).unwrap()
            .build()
            .unwrap()];

    let (map_texture, palette_texture) = {
        use vulkano::image::immutable::ImmutableImage;
        use vulkano::image::Dimensions;
        use vulkano::format::{R8Unorm, B5G5R5A1UnormPack16};
        (ImmutableImage::new(device.clone(),
                             Dimensions::Dim2d {
                                 width: art::MAP_WIDTH,
                                 height: art::MAP_HEIGHT,
                             },
                             R8Unorm,
                             Some(queue.family()))
            .unwrap(),
         ImmutableImage::new(device.clone(),
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
        (Sampler::new(device.clone(),
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
         Sampler::new(device.clone(),
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

    //mod art_pipeline_layout {
    //    pipeline_layout!{
    //        set0: {
    //            map: CombinedImageSampler,
    //            palette: CombinedImageSampler,
    //            global: UniformBuffer<::art::aa_fs::ty::Globals>,
    //            pc: UniformBuffer<::art::aa_fs::ty::PaletteCycles>,
    //            translations: UniformBuffer<::art::aa_fs::ty::Translations>,
    //            distortions: UniformBuffer<::art::aa_fs::ty::Distortions>
    //        }
    //    }
    //}
    //let art_pipeline_layout = art_pipeline_layout::CustomPipeline::new(&device).unwrap();

    let art_pipeline = {
        let dim = map_texture.dimensions().width_height();
        Arc::new(vulkano::pipeline::GraphicsPipeline::new(device.clone(),
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
        //layout: &art_pipeline_layout,
        render_pass: vulkano::framebuffer::Subpass::from(renderpass.clone(), 0).unwrap(),
      }
        ).unwrap())
    };

    let layer_sets = layer_buffers.iter()
        .map(|buffers| {
            Arc::new(simple_descriptor_set!(art_pipeline.clone(), 0, {
                map: (map_texture.clone(), map_sampler.clone()),
                palette: (palette_texture.clone(), palette_sampler.clone()),
                global: global_buffer.clone(),
                pc: buffers.palette_cycles.clone(),
                translations: buffers.translations.clone(),
                distortions: buffers.distortions.clone(),
            }))
        })
        .collect::<Vec<_>>();


    //let descriptor_pool = vulkano::descriptor::descriptor_set::DescriptorPool::new(&device);
    //mod pipeline_layout {
    //    pipeline_layout! {
    //        set0: {
    //            bg3: CombinedImageSampler,
    //            bg4: CombinedImageSampler
    //        }
    //    }
    //}

    //let pipeline_layout = pipeline_layout::CustomPipeline::new(&device).unwrap();
    let compose_pipeline = Arc::new(
        vulkano::pipeline::GraphicsPipeline::new(device.clone(), vulkano::pipeline::GraphicsPipelineParams {
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
            //layout: &pipeline_layout,
            render_pass: vulkano::framebuffer::Subpass::from(renderpass.clone(), 0).unwrap(),
        }).unwrap()
    );
    //let set = pipeline_layout::set0::Set::new(&descriptor_pool,
    //                                          &pipeline_layout,
    //                                          &pipeline_layout::set0::Descriptors {
    //                                              bg3: (&map_sampler, &art_tex[0]),
    //                                              bg4: (&map_sampler, &art_tex[1]),
    //                                          });
    let set = Arc::new(simple_descriptor_set!(compose_pipeline.clone(), 0, {
        bg3: (art_tex[0].clone(), map_sampler.clone()),
        bg4: (art_tex[1].clone(), map_sampler.clone()),
    }));

    let art_pipeline = {
        let dim = map_texture.dimensions().width_height();
        Arc::new(vulkano::pipeline::GraphicsPipeline::new(device.clone(),
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
        //layout: &art_pipeline_layout,
        render_pass: vulkano::framebuffer::Subpass::from(renderpass.clone(), 0).unwrap(),
      }
  ).unwrap())
    };

    /*let pipeline = {
      vulkano::pipeline::GraphicsPipeline::new(device.clone(), vulkano::pipeline::GraphicsPipelineParams {
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
        //layout: &pipeline_layout,
        render_pass: vulkano::framebuffer::Subpass::from(renderpass.clone(), 0).unwrap(),
      }).unwrap()
    };*/

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
    use vulkano::command_buffer::CommandBufferBuilder;
    let build_command_buffers = |state: vulkano::command_buffer::DynamicState, framebuffers: &Vec<Arc<vulkano::framebuffer::Framebuffer<_, _>>>| framebuffers.iter().map(|framebuffer| {
        use vulkano::buffer::BufferSlice;
        use vulkano::command_buffer::{AutoCommandBufferBuilder, DynamicState};
        let mut command_buffer = AutoCommandBufferBuilder::new(device.clone(), queue.family()).unwrap();
        for ((buffers, layer_framebuffer), layer_set) in layer_buffers.iter().zip(layer_framebuffers.iter()).zip(layer_sets.iter()) {
            command_buffer = command_buffer
                .copy_buffer_to_image(buffers.map.clone(), map_texture.clone()).unwrap()
                //.copy_buffer_to_image(BufferSlice::from(&buffers.map), &map_texture, 0, 0 .. 1, [0, 0, 0],
                //    [map_texture.dimensions().width(), map_texture.dimensions().height(), 1])
                .copy_buffer_to_image(buffers.palette.clone(), palette_texture.clone()).unwrap()
                //.copy_buffer_to_image(BufferSlice::from(&buffers.palette),
                //    &palette_texture, 0, 0 .. 1, [0, 0, 0],
                //    [palette_texture.dimensions().width(), palette_texture.dimensions().height(), 1])
                // Layer 1
                .begin_render_pass(layer_framebuffer.clone(), false,
                    vec![
                        [0.0, 0.0, 0.0, 0.0].into()
                    ]
                ).unwrap()
                .draw(art_pipeline.clone(), DynamicState::none(),
                    vertex_buffer.clone(), layer_set.clone(), ()).unwrap()
                .end_render_pass().unwrap();
        }
        command_buffer
            // Compose
            .begin_render_pass(framebuffer.clone(), false,
                vec![
                    [0.0, 0.0, 0.0, 1.0].into()
                ]
            ).unwrap()
            .draw(compose_pipeline.clone(), state.clone(),
                vertex_buffer.clone(), set.clone(), ()).unwrap()
            .end_render_pass().unwrap()
            
            .build()
    }).collect::<Vec<_>>();

    let mut command_buffers = build_command_buffers(state, &framebuffers);

    let mut previous_frame = Box::new(vulkano::sync::now(device.clone())) as Box<vulkano::sync::GpuFuture>;

    let mut running = true;

    while running {
        previous_frame.cleanup_finished();
        
        {
            let mut mapping = global_buffer.write().unwrap();
            mapping.time = get_time();
        }

        let (image_num, acquire_future) = vulkano::swapchain::acquire_next_image(swapchain.clone(),
            Duration::new(2, 0)).unwrap();
        
        let future = previous_frame.join(acquire_future)
            .then_execute(queue.clone(), command_buffers[image_num]).unwrap()
            .then_swapchain_present(queue.clone(), swapchain.clone(), image_num)
            .then_signal_fence_and_flush().unwrap();
        
        previous_frame = Box::new(future) as Box<_>;
        //submissions.push(vulkano::command_buffer::submit(&command_buffers[image_num], &queue).unwrap());
        //swapchain.present(&queue, image_num).unwrap();

        let mut fullscreen_toggle = fullscreen;
        let mut rebuild_swapchain = false;
        let mut use_old_swapchain = false;

        events_loop.poll_events(|event| {
            use winit::{WindowEvent};
            match event {
                winit::Event::WindowEvent { event, .. } => match event {
                    WindowEvent::Closed => running = false,
                    _ => (),
                }
            }
        });
        //for ev in window.window().poll_events() {
        //    use winit::{Event, VirtualKeyCode, ElementState, MouseScrollDelta, TouchPhase};
        //    match ev {
        //        Event::Closed => break 'run,
        //        Event::KeyboardInput(_, _, Some(VirtualKeyCode::Escape)) => break 'run,
        //        Event::KeyboardInput(ElementState::Pressed, _, Some(VirtualKeyCode::F11)) => fullscreen_toggle = !fullscreen_toggle,
        //        Event::Resized(width, height) => {
        //            dimensions = [width, height];
        //            rebuild_swapchain = true;
        //            use_old_swapchain = true;
        //        }
        //        Event::MouseWheel(MouseScrollDelta::LineDelta(_, d), TouchPhase::Moved) => {
        //            change_image(d as i16)
        //        }
        //        Event::KeyboardInput(ElementState::Pressed, _, Some(VirtualKeyCode::Up)) => {
        //            change_image(1)
        //        }
        //        Event::KeyboardInput(ElementState::Pressed, _, Some(VirtualKeyCode::Down)) => {
        //            change_image(-1)
        //        }
        //        _ => (),
        //    }
        //}
        if fullscreen != fullscreen_toggle {
          fullscreen = fullscreen_toggle;
          if !fullscreen {
            dimensions = [art::MAP_WIDTH, art::MAP_HEIGHT];
          }
          window = build_window(&events_loop, &instance, fullscreen, dimensions);
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
                let sc = create_swapchain(device.clone(),
                    window.surface().clone(),
                    &caps,
                    vulkano::sync::SharingMode::from(&queue),
                    dimensions,
                    old_swapchain);
                
                let mut framebuffers = create_framebuffers(renderpass.clone(), sc.1);
                (sc.0, framebuffers)
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

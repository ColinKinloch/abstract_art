extern crate vulkano_shaders;

fn main() {
    vulkano_shaders::build_glsl_shaders([
        ("src/art/draw.glslv", vulkano_shaders::ShaderType::Vertex),
        ("src/art/abstract_art.glslf", vulkano_shaders::ShaderType::Fragment),
        ("src/art/compose.glslf", vulkano_shaders::ShaderType::Fragment),
    ].iter().cloned());
}

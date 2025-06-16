use crate::scenes::*;
use clap::{Args, ValueEnum};
use raytracer_core::{camera::Camera, scene::Scene};
use raytracer_cpu_renderer::{
    brdf::Brdf,
    brdfs::{disney::DisneyBrdf, lambertian::LambertianBrdf},
    renderer::{CpuRenderer, CpuRendererConfig},
};
use std::{fs::File, io::BufWriter, time::Instant};

#[derive(Args, Debug)]
#[command(about = "Render a scene preset using given options")]
#[command(arg_required_else_help = true)]
pub struct RenderCommand {
    device: Device,

    #[arg(short = 'w', long)]
    image_width: u32,
    #[arg(short = 'h', long)]
    image_height: u32,

    #[arg(long, default_value = "disney")]
    brdf: BrdfName,

    #[arg(short = 's', long)]
    sample_per_pixel: u32,
    #[arg(short = 'b', long, default_value = "8")]
    max_ray_bounces: u32,

    #[arg(long, default_value = "1.0")]
    exposure: f32,
    #[arg(long, default_value = "2.2")]
    gamma: f32,

    #[arg(short = 'p', long, default_value = "cornell-box")]
    scene_preset: ScenePreset,

    #[arg(short = 'o', long, default_value = "./output.png")]
    output: String,
}

#[derive(ValueEnum, Debug, Clone, Copy)]
pub enum Device {
    Cpu,
    Gpu,
}

#[derive(ValueEnum, Debug, Clone, Copy)]
pub enum BrdfName {
    Disney,
    Lambertian,
}

#[derive(ValueEnum, Debug, Clone, Copy)]
pub enum ScenePreset {
    CornellBox,
}

pub fn handle_render_command(cmd: RenderCommand) -> Result<(), Box<dyn std::error::Error>> {
    let (scene, camera) = match cmd.scene_preset {
        ScenePreset::CornellBox => cornell_box::create_cornell_box(),
    };
    let brdf: Box<dyn Brdf> = match cmd.brdf {
        BrdfName::Disney => Box::new(DisneyBrdf) as Box<dyn Brdf>,
        BrdfName::Lambertian => Box::new(LambertianBrdf) as Box<dyn Brdf>,
    };

    let frame_buffer = match cmd.device {
        Device::Cpu => render_cpu(scene, camera, &cmd, brdf),
        Device::Gpu => render_gpu(scene, camera, &cmd, brdf),
    };

    let file = File::create(cmd.output)?;
    let w = BufWriter::new(file);

    let mut encoder = png::Encoder::new(w, cmd.image_width, cmd.image_height);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);

    let mut writer = encoder.write_header()?;
    writer.write_image_data(&frame_buffer)?;

    Ok(())
}

fn render_cpu(scene: Scene, camera: Camera, cmd: &RenderCommand, brdf: Box<dyn Brdf>) -> Vec<u8> {
    println!("rendering the {} with CPU", scene.name());

    let started_at = Instant::now();
    let renderer = CpuRenderer::new(CpuRendererConfig {
        screen_width: cmd.image_width,
        screen_height: cmd.image_height,
        sample_per_pixel: cmd.sample_per_pixel,
        max_ray_bounces: cmd.max_ray_bounces,
        exposure: cmd.exposure,
        gamma: cmd.gamma,
    });
    let frame_buffer = renderer.render(&scene, &camera, brdf.as_ref());

    let finished_at = Instant::now();
    let render_time = finished_at.duration_since(started_at);

    println!("render took {:.2} seconds", render_time.as_secs_f32());

    frame_buffer
}

fn render_gpu(
    _scene: Scene,
    _camera: Camera,
    _cmd: &RenderCommand,
    _brdf: Box<dyn Brdf>,
) -> Vec<u8> {
    panic!("GPU is not supported yet");
}

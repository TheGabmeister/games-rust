use macroquad::prelude::*;
use macroquad::window::miniquad::*;

use crate::shaders;

pub struct PostFxPipeline {
    rt_a: RenderTarget,
    rt_b: RenderTarget,
    rt_bloom_half: RenderTarget,
    rt_bloom_blur: RenderTarget,
    mat_bloom_down: Material,
    mat_blur_h: Material,
    mat_blur_v: Material,
    mat_bloom_combine: Material,
    mat_chromatic: Material,
    mat_wave: Material,
    mat_crt: Material,
    mat_vignette: Material,
    width: u32,
    height: u32,
    pub bloom_enabled: bool,
    pub chromatic_enabled: bool,
    pub wave_enabled: bool,
    pub crt_enabled: bool,
    pub vignette_enabled: bool,
}

impl PostFxPipeline {
    pub fn new(w: u32, h: u32) -> Self {
        let rt_a = render_target(w, h);
        rt_a.texture.set_filter(FilterMode::Linear);
        let rt_b = render_target(w, h);
        rt_b.texture.set_filter(FilterMode::Linear);

        let half_w = w / 2;
        let half_h = h / 2;
        let rt_bloom_half = render_target(half_w, half_h);
        rt_bloom_half.texture.set_filter(FilterMode::Linear);
        let rt_bloom_blur = render_target(half_w, half_h);
        rt_bloom_blur.texture.set_filter(FilterMode::Linear);

        let mat_bloom_down = load_material(
            ShaderSource::Glsl { vertex: shaders::VERTEX, fragment: shaders::BLOOM_DOWNSAMPLE_FRAG },
            MaterialParams {
                uniforms: vec![UniformDesc::new("threshold", UniformType::Float1)],
                ..Default::default()
            },
        ).unwrap();

        let mat_blur_h = load_material(
            ShaderSource::Glsl { vertex: shaders::VERTEX, fragment: shaders::BLUR_H_FRAG },
            MaterialParams {
                uniforms: vec![UniformDesc::new("texel_size", UniformType::Float1)],
                ..Default::default()
            },
        ).unwrap();

        let mat_blur_v = load_material(
            ShaderSource::Glsl { vertex: shaders::VERTEX, fragment: shaders::BLUR_V_FRAG },
            MaterialParams {
                uniforms: vec![UniformDesc::new("texel_size", UniformType::Float1)],
                ..Default::default()
            },
        ).unwrap();

        let additive_pipeline = PipelineParams {
            color_blend: Some(BlendState::new(
                Equation::Add,
                BlendFactor::One,
                BlendFactor::One,
            )),
            ..Default::default()
        };

        let mat_bloom_combine = load_material(
            ShaderSource::Glsl { vertex: shaders::VERTEX, fragment: shaders::BLOOM_COMBINE_FRAG },
            MaterialParams {
                pipeline_params: additive_pipeline,
                ..Default::default()
            },
        ).unwrap();

        let mat_chromatic = load_material(
            ShaderSource::Glsl { vertex: shaders::VERTEX, fragment: shaders::CHROMATIC_FRAG },
            MaterialParams {
                uniforms: vec![UniformDesc::new("aberration", UniformType::Float1)],
                ..Default::default()
            },
        ).unwrap();

        let mat_wave = load_material(
            ShaderSource::Glsl { vertex: shaders::VERTEX, fragment: shaders::WAVE_FRAG },
            MaterialParams {
                uniforms: vec![
                    UniformDesc::new("time", UniformType::Float1),
                    UniformDesc::new("intensity", UniformType::Float1),
                ],
                ..Default::default()
            },
        ).unwrap();

        let mat_crt = load_material(
            ShaderSource::Glsl { vertex: shaders::VERTEX, fragment: shaders::CRT_FRAG },
            MaterialParams {
                uniforms: vec![UniformDesc::new("time", UniformType::Float1)],
                ..Default::default()
            },
        ).unwrap();

        let mat_vignette = load_material(
            ShaderSource::Glsl { vertex: shaders::VERTEX, fragment: shaders::VIGNETTE_FRAG },
            MaterialParams {
                uniforms: vec![UniformDesc::new("strength", UniformType::Float1)],
                ..Default::default()
            },
        ).unwrap();

        Self {
            rt_a, rt_b, rt_bloom_half, rt_bloom_blur,
            mat_bloom_down, mat_blur_h, mat_blur_v, mat_bloom_combine,
            mat_chromatic, mat_wave, mat_crt, mat_vignette,
            width: w, height: h,
            bloom_enabled: true,
            chromatic_enabled: true,
            wave_enabled: true,
            crt_enabled: true,
            vignette_enabled: true,
        }
    }

    /// Apply the full post-processing chain. `source` is the scene render target.
    /// The final result is drawn to the screen.
    pub fn apply(&self, source: &RenderTarget, time: f32) {
        let w = self.width as f32;
        let h = self.height as f32;
        let half_w = (self.width / 2) as f32;
        let half_h = (self.height / 2) as f32;

        // Track which render target holds the current result.
        // We alternate between rt_a and rt_b. `in_a` means the latest result is in rt_a.
        let mut in_a: bool;

        // --- BLOOM ---
        if self.bloom_enabled {
            self.mat_bloom_down.set_uniform("threshold", 0.6_f32);
            self.fullscreen_pass(&source.texture, &self.rt_bloom_half, &self.mat_bloom_down, half_w, half_h);
            self.mat_blur_h.set_uniform("texel_size", 1.0 / half_w);
            self.fullscreen_pass(&self.rt_bloom_half.texture, &self.rt_bloom_blur, &self.mat_blur_h, half_w, half_h);
            self.mat_blur_v.set_uniform("texel_size", 1.0 / half_h);
            self.fullscreen_pass(&self.rt_bloom_blur.texture, &self.rt_bloom_half, &self.mat_blur_v, half_w, half_h);
            self.fullscreen_pass_no_material(&source.texture, &self.rt_a, w, h);
            self.fullscreen_pass(&self.rt_bloom_half.texture, &self.rt_a, &self.mat_bloom_combine, w, h);
        } else {
            self.fullscreen_pass_no_material(&source.texture, &self.rt_a, w, h);
        }
        in_a = true;

        // --- CHROMATIC ABERRATION ---
        if self.chromatic_enabled {
            self.mat_chromatic.set_uniform("aberration", 0.003_f32);
            if in_a {
                self.fullscreen_pass(&self.rt_a.texture, &self.rt_b, &self.mat_chromatic, w, h);
            } else {
                self.fullscreen_pass(&self.rt_b.texture, &self.rt_a, &self.mat_chromatic, w, h);
            }
            in_a = !in_a;
        }

        // --- WAVE DISTORTION ---
        if self.wave_enabled {
            self.mat_wave.set_uniform("time", time);
            self.mat_wave.set_uniform("intensity", 0.002_f32);
            if in_a {
                self.fullscreen_pass(&self.rt_a.texture, &self.rt_b, &self.mat_wave, w, h);
            } else {
                self.fullscreen_pass(&self.rt_b.texture, &self.rt_a, &self.mat_wave, w, h);
            }
            in_a = !in_a;
        }

        // --- CRT ---
        if self.crt_enabled {
            self.mat_crt.set_uniform("time", time);
            if in_a {
                self.fullscreen_pass(&self.rt_a.texture, &self.rt_b, &self.mat_crt, w, h);
            } else {
                self.fullscreen_pass(&self.rt_b.texture, &self.rt_a, &self.mat_crt, w, h);
            }
            in_a = !in_a;
        }

        // --- VIGNETTE + COLOR GRADING (final to screen) ---
        let final_source = if in_a { &self.rt_a.texture } else { &self.rt_b.texture };
        if self.vignette_enabled {
            self.mat_vignette.set_uniform("strength", 0.3_f32);
            self.draw_to_screen(final_source, &self.mat_vignette);
        } else {
            self.draw_to_screen_no_material(final_source);
        }
    }

    fn camera_for_rt(rt: &RenderTarget, w: f32, h: f32) -> Camera2D {
        Camera2D::from_display_rect(Rect::new(0.0, 0.0, w, h)).with_render_target(rt)
    }

    fn fullscreen_pass(&self, source: &Texture2D, dest: &RenderTarget, material: &Material, w: f32, h: f32) {
        let cam = Self::camera_for_rt(dest, w, h);
        set_camera(&cam);
        gl_use_material(material);
        draw_texture_ex(source, 0.0, 0.0, WHITE, DrawTextureParams {
            dest_size: Some(vec2(w, h)),
            ..Default::default()
        });
        gl_use_default_material();
    }

    fn fullscreen_pass_no_material(&self, source: &Texture2D, dest: &RenderTarget, w: f32, h: f32) {
        let cam = Self::camera_for_rt(dest, w, h);
        set_camera(&cam);
        clear_background(BLACK);
        draw_texture_ex(source, 0.0, 0.0, WHITE, DrawTextureParams {
            dest_size: Some(vec2(w, h)),
            ..Default::default()
        });
    }

    fn draw_to_screen(&self, source: &Texture2D, material: &Material) {
        set_default_camera();
        gl_use_material(material);
        draw_texture_ex(source, 0.0, 0.0, WHITE, DrawTextureParams {
            dest_size: Some(vec2(screen_width(), screen_height())),
            ..Default::default()
        });
        gl_use_default_material();
    }

    fn draw_to_screen_no_material(&self, source: &Texture2D) {
        set_default_camera();
        draw_texture_ex(source, 0.0, 0.0, WHITE, DrawTextureParams {
            dest_size: Some(vec2(screen_width(), screen_height())),
            ..Default::default()
        });
    }
}

trait WithRenderTarget {
    fn with_render_target(self, rt: &RenderTarget) -> Self;
}

impl WithRenderTarget for Camera2D {
    fn with_render_target(mut self, rt: &RenderTarget) -> Self {
        self.render_target = Some(rt.clone());
        self
    }
}

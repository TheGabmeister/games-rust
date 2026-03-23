use macroquad::prelude::*;

use crate::shaders;

pub struct TransitionSystem {
    materials: [Material; 4],
}

impl TransitionSystem {
    pub fn new() -> Self {
        let make = |frag: &str| {
            load_material(
                ShaderSource::Glsl {
                    vertex: shaders::VERTEX,
                    fragment: frag,
                },
                MaterialParams {
                    uniforms: vec![UniformDesc::new("progress", UniformType::Float1)],
                    textures: vec!["Texture2".to_string()],
                    ..Default::default()
                },
            )
            .unwrap()
        };

        Self {
            materials: [
                make(shaders::TRANSITION_DISSOLVE_FRAG),
                make(shaders::TRANSITION_RADIAL_FRAG),
                make(shaders::TRANSITION_PIXELATE_FRAG),
                make(shaders::TRANSITION_SLIDE_FRAG),
            ],
        }
    }

    /// Composite current and next scene render targets using the given transition.
    /// `transition_idx` selects which effect (mod 4).
    /// `progress` is 0.0 → 1.0.
    /// Draws the result into `dest`.
    pub fn apply(
        &self,
        current: &RenderTarget,
        next: &RenderTarget,
        dest: &RenderTarget,
        transition_idx: usize,
        progress: f32,
    ) {
        let idx = transition_idx % 4;
        let mat = &self.materials[idx];

        // Smoothstep the progress for nicer easing
        let p = smoothstep(progress);
        mat.set_uniform("progress", p);
        mat.set_texture("Texture2", next.texture.clone());

        let w = screen_width();
        let h = screen_height();

        let cam = Camera2D::from_display_rect(Rect::new(0.0, 0.0, w, h));
        set_camera(&Camera2D {
            render_target: Some(dest.clone()),
            ..cam
        });

        gl_use_material(mat);
        draw_texture_ex(&current.texture, 0.0, 0.0, WHITE, DrawTextureParams {
            dest_size: Some(vec2(w, h)),
            ..Default::default()
        });
        gl_use_default_material();
    }
}

fn smoothstep(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

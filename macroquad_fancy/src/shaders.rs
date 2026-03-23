/// Common vertex shader used by all fullscreen post-processing and transition passes.
pub const VERTEX: &str = r#"#version 100
attribute vec3 position;
attribute vec2 texcoord;
attribute vec4 color0;

varying lowp vec2 uv;
varying lowp vec4 color;

uniform mat4 Model;
uniform mat4 Projection;

void main() {
    gl_Position = Projection * Model * vec4(position, 1);
    color = color0 / 255.0;
    uv = texcoord;
}
"#;

// ---------------------------------------------------------------------------
// Post-processing fragment shaders
// ---------------------------------------------------------------------------

pub const BLOOM_DOWNSAMPLE_FRAG: &str = r#"#version 100
precision lowp float;

varying vec2 uv;
uniform sampler2D Texture;
uniform float threshold;

void main() {
    vec4 col = texture2D(Texture, uv);
    float brightness = dot(col.rgb, vec3(0.2126, 0.7152, 0.0722));
    gl_FragColor = col * step(threshold, brightness);
}
"#;

pub const BLUR_H_FRAG: &str = r#"#version 100
precision lowp float;

varying vec2 uv;
uniform sampler2D Texture;
uniform float texel_size;

void main() {
    vec4 sum = vec4(0.0);
    sum += texture2D(Texture, uv + vec2(-4.0 * texel_size, 0.0)) * 0.0162;
    sum += texture2D(Texture, uv + vec2(-3.0 * texel_size, 0.0)) * 0.0540;
    sum += texture2D(Texture, uv + vec2(-2.0 * texel_size, 0.0)) * 0.1218;
    sum += texture2D(Texture, uv + vec2(-1.0 * texel_size, 0.0)) * 0.1859;
    sum += texture2D(Texture, uv) * 0.1974;
    sum += texture2D(Texture, uv + vec2( 1.0 * texel_size, 0.0)) * 0.1859;
    sum += texture2D(Texture, uv + vec2( 2.0 * texel_size, 0.0)) * 0.1218;
    sum += texture2D(Texture, uv + vec2( 3.0 * texel_size, 0.0)) * 0.0540;
    sum += texture2D(Texture, uv + vec2( 4.0 * texel_size, 0.0)) * 0.0162;
    gl_FragColor = sum;
}
"#;

pub const BLUR_V_FRAG: &str = r#"#version 100
precision lowp float;

varying vec2 uv;
uniform sampler2D Texture;
uniform float texel_size;

void main() {
    vec4 sum = vec4(0.0);
    sum += texture2D(Texture, uv + vec2(0.0, -4.0 * texel_size)) * 0.0162;
    sum += texture2D(Texture, uv + vec2(0.0, -3.0 * texel_size)) * 0.0540;
    sum += texture2D(Texture, uv + vec2(0.0, -2.0 * texel_size)) * 0.1218;
    sum += texture2D(Texture, uv + vec2(0.0, -1.0 * texel_size)) * 0.1859;
    sum += texture2D(Texture, uv) * 0.1974;
    sum += texture2D(Texture, uv + vec2(0.0,  1.0 * texel_size)) * 0.1859;
    sum += texture2D(Texture, uv + vec2(0.0,  2.0 * texel_size)) * 0.1218;
    sum += texture2D(Texture, uv + vec2(0.0,  3.0 * texel_size)) * 0.0540;
    sum += texture2D(Texture, uv + vec2(0.0,  4.0 * texel_size)) * 0.0162;
    gl_FragColor = sum;
}
"#;

pub const BLOOM_COMBINE_FRAG: &str = r#"#version 100
precision lowp float;

varying vec2 uv;
uniform sampler2D Texture;

void main() {
    gl_FragColor = texture2D(Texture, uv);
}
"#;

pub const CHROMATIC_FRAG: &str = r#"#version 100
precision lowp float;

varying vec2 uv;
uniform sampler2D Texture;
uniform float aberration;

void main() {
    float r = texture2D(Texture, uv + vec2(aberration, 0.0)).r;
    float g = texture2D(Texture, uv).g;
    float b = texture2D(Texture, uv - vec2(aberration, 0.0)).b;
    float a = texture2D(Texture, uv).a;
    gl_FragColor = vec4(r, g, b, a);
}
"#;

pub const WAVE_FRAG: &str = r#"#version 100
precision lowp float;

varying vec2 uv;
uniform sampler2D Texture;
uniform float time;
uniform float intensity;

void main() {
    vec2 distorted = uv;
    distorted.y += sin(uv.y * 30.0 + time * 2.0) * intensity;
    distorted.x += sin(uv.x * 20.0 + time * 1.5) * intensity * 0.5;
    gl_FragColor = texture2D(Texture, distorted);
}
"#;

pub const CRT_FRAG: &str = r#"#version 100
precision lowp float;

varying vec2 uv;
uniform sampler2D Texture;
uniform float time;

vec2 barrel(vec2 coord) {
    vec2 cc = coord - 0.5;
    float r2 = dot(cc, cc);
    return coord + cc * r2 * 0.15;
}

void main() {
    vec2 crt_uv = barrel(uv);
    if (crt_uv.x < 0.0 || crt_uv.x > 1.0 || crt_uv.y < 0.0 || crt_uv.y > 1.0) {
        gl_FragColor = vec4(0.0);
        return;
    }
    vec3 col = texture2D(Texture, crt_uv).rgb;
    // scanlines
    float scanline = 0.95 + 0.05 * cos(3.14159 * (crt_uv.y + 0.008 * time) * 480.0);
    // grille
    float grille = 0.85 + 0.15 * clamp(1.5 * cos(3.14159 * crt_uv.x * 1280.0), 0.0, 1.0);
    col *= scanline * grille * 1.2;
    gl_FragColor = vec4(col, 1.0);
}
"#;

pub const VIGNETTE_FRAG: &str = r#"#version 100
precision lowp float;

varying vec2 uv;
uniform sampler2D Texture;
uniform float strength;

void main() {
    vec3 col = texture2D(Texture, uv).rgb;
    // vignette
    float vignette = uv.x * uv.y * (1.0 - uv.x) * (1.0 - uv.y);
    vignette = clamp(pow(16.0 * vignette, strength), 0.0, 1.0);
    col *= vignette;
    // slight color grading: warm highlights, cool shadows
    col.r *= 1.05;
    col.b *= 0.95;
    gl_FragColor = vec4(col, 1.0);
}
"#;

// ---------------------------------------------------------------------------
// Transition fragment shaders
// ---------------------------------------------------------------------------

pub const TRANSITION_DISSOLVE_FRAG: &str = r#"#version 100
precision lowp float;

varying vec2 uv;
uniform sampler2D Texture;
uniform sampler2D Texture2;
uniform float progress;

float hash(vec2 p) {
    return fract(sin(dot(p, vec2(12.9898, 78.233))) * 43758.5453);
}

void main() {
    float n = hash(floor(uv * 80.0));
    float t = smoothstep(progress - 0.05, progress + 0.05, n);
    vec4 a = texture2D(Texture, uv);
    vec4 b = texture2D(Texture2, uv);
    gl_FragColor = mix(a, b, t);
}
"#;

pub const TRANSITION_RADIAL_FRAG: &str = r#"#version 100
precision lowp float;

varying vec2 uv;
uniform sampler2D Texture;
uniform sampler2D Texture2;
uniform float progress;

void main() {
    vec2 center = uv - 0.5;
    float angle = atan(center.y, center.x) + 3.14159;
    float t = smoothstep(progress * 6.28318 - 0.2, progress * 6.28318, angle);
    vec4 a = texture2D(Texture, uv);
    vec4 b = texture2D(Texture2, uv);
    gl_FragColor = mix(a, b, t);
}
"#;

pub const TRANSITION_PIXELATE_FRAG: &str = r#"#version 100
precision lowp float;

varying vec2 uv;
uniform sampler2D Texture;
uniform sampler2D Texture2;
uniform float progress;

void main() {
    float p = progress < 0.5 ? progress * 2.0 : (1.0 - progress) * 2.0;
    float pixels = mix(512.0, 8.0, p);
    vec2 pixelated = floor(uv * pixels) / pixels;
    if (progress < 0.5) {
        gl_FragColor = texture2D(Texture, pixelated);
    } else {
        gl_FragColor = texture2D(Texture2, pixelated);
    }
}
"#;

pub const TRANSITION_SLIDE_FRAG: &str = r#"#version 100
precision lowp float;

varying vec2 uv;
uniform sampler2D Texture;
uniform sampler2D Texture2;
uniform float progress;

void main() {
    if (uv.x < 1.0 - progress) {
        gl_FragColor = texture2D(Texture, vec2(uv.x + progress, uv.y));
    } else {
        gl_FragColor = texture2D(Texture2, vec2(uv.x - 1.0 + progress, uv.y));
    }
}
"#;

// ---------------------------------------------------------------------------
// Scene-specific shaders
// ---------------------------------------------------------------------------

pub const VORONOI_FRAG: &str = r#"#version 100
precision mediump float;

varying vec2 uv;
uniform sampler2D Texture;
uniform vec4 seeds[15];
uniform vec4 seed_colors[30];
uniform vec2 resolution;

void main() {
    vec2 pos = uv * resolution;
    float min_dist = 99999.0;
    float second_dist = 99999.0;
    int closest = 0;

    for (int i = 0; i < 15; i++) {
        // Each vec4 packs two seed positions: .xy and .zw
        vec2 s1 = seeds[i].xy;
        float d1 = distance(pos, s1);
        if (d1 < min_dist) {
            second_dist = min_dist;
            min_dist = d1;
            closest = i * 2;
        } else if (d1 < second_dist) {
            second_dist = d1;
        }

        vec2 s2 = seeds[i].zw;
        float d2 = distance(pos, s2);
        if (d2 < min_dist) {
            second_dist = min_dist;
            min_dist = d2;
            closest = i * 2 + 1;
        } else if (d2 < second_dist) {
            second_dist = d2;
        }
    }

    vec4 cell_color = seed_colors[closest];
    float edge = smoothstep(0.0, 8.0, second_dist - min_dist);
    gl_FragColor = cell_color * edge;
}
"#;

#[allow(dead_code)]
pub const FIRE_SHIMMER_FRAG: &str = r#"#version 100
precision lowp float;

varying vec2 uv;
uniform sampler2D Texture;
uniform float time;

void main() {
    vec2 distorted = uv;
    float strength = (1.0 - uv.y) * 0.003;
    distorted.x += sin(uv.y * 40.0 + time * 5.0) * strength;
    gl_FragColor = texture2D(Texture, distorted);
}
"#;

/// Passthrough fragment — used with additive blend pipeline.
pub const PASSTHROUGH_FRAG: &str = r#"#version 100
precision lowp float;

varying vec2 uv;
varying vec4 color;
uniform sampler2D Texture;

void main() {
    gl_FragColor = texture2D(Texture, uv) * color;
}
"#;

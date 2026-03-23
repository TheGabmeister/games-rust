use macroquad::prelude::*;

pub struct Palette {
    pub background: Color,
    pub colors: &'static [Color],
}

// Scene 1: Starfield Warp
pub const STARFIELD: Palette = Palette {
    background: color_from_hex(0x000011),
    colors: &[
        color_from_hex(0xAABBFF),
        color_from_hex(0xCCDDFF),
        color_from_hex(0xFFFFFF),
    ],
};

// Scene 2: Fire Particles
pub const FIRE: Palette = Palette {
    background: color_from_hex(0x000000),
    colors: &[
        color_from_hex(0xFFFF44),
        color_from_hex(0xFF8800),
        color_from_hex(0xFF4400),
        color_from_hex(0xCC1100),
        color_from_hex(0x440000),
    ],
};

// Scene 3: Spirograph
pub const SPIROGRAPH: Palette = Palette {
    background: color_from_hex(0x0A0A2E),
    colors: &[
        color_from_hex(0x00FFCC),
        color_from_hex(0xFF00FF),
        color_from_hex(0xFFD700),
        color_from_hex(0x00AAFF),
    ],
};

// Scene 4: Moire Patterns
pub const MOIRE: Palette = Palette {
    background: color_from_hex(0x000000),
    colors: &[
        Color::new(1.0, 1.0, 1.0, 0.4),
        Color::new(1.0, 1.0, 1.0, 0.35),
    ],
};

// Scene 5: Aurora Borealis
pub const AURORA: Palette = Palette {
    background: color_from_hex(0x050520),
    colors: &[
        color_from_hex(0x00FF88),
        color_from_hex(0x00CC66),
        color_from_hex(0x4488FF),
        color_from_hex(0x2244CC),
        color_from_hex(0x8844FF),
        color_from_hex(0x6622CC),
    ],
};

// Scene 6: Voronoi Shatter
pub const VORONOI: Palette = Palette {
    background: color_from_hex(0x111111),
    colors: &[
        color_from_hex(0xFF3366),
        color_from_hex(0x33CCFF),
        color_from_hex(0xFFCC00),
        color_from_hex(0x66FF66),
        color_from_hex(0xCC66FF),
        color_from_hex(0xFF6633),
    ],
};

const fn color_from_hex(hex: u32) -> Color {
    Color::new(
        ((hex >> 16) & 0xFF) as f32 / 255.0,
        ((hex >> 8) & 0xFF) as f32 / 255.0,
        (hex & 0xFF) as f32 / 255.0,
        1.0,
    )
}

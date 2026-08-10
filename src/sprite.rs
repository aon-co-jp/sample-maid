//! 手描きのドット絵パターンをRGBA8テクスチャへ焼き込む(ユーザー指示、
//! 2026-08-10「PNGドット絵スプライト(事前生成)」への対応)。`font.rs`と
//! 同じ「文字→ドットパターン→RGBA8」という考え方を、キャラクター
//! スプライト(メイド・トラさん・魔女・雑魚敵)に適用したもの。
//!
//! パターンは1行1文字列の配列(全行同じ長さ)で表し、各文字を
//! `palette`関数でRGBA8色へマッピングする。`.`は背景色(スプライトの
//! 縁取り・下地)として扱う。実際の画像ファイル(PNG)を用意する代わりに、
//! このプログラム自身がドット絵を生成する——アセット制作ツールが無い
//! Phase 0の制約下での実務的な折衷案(正直な開示)。

use directx_graphics_vulkan::{Rgba8, TextureRgba8};

pub fn rasterize(pattern: &[&str], palette: impl Fn(char) -> Rgba8, scale: usize) -> TextureRgba8 {
    let rows = pattern.len();
    let cols = pattern[0].chars().count();
    let width = cols * scale;
    let height = rows * scale;
    let mut pixels = vec![palette('.'); width * height];

    for (row, line) in pattern.iter().enumerate() {
        for (col, ch) in line.chars().enumerate() {
            let color = palette(ch);
            for dy in 0..scale {
                for dx in 0..scale {
                    let x = col * scale + dx;
                    let y = row * scale + dy;
                    pixels[y * width + x] = color;
                }
            }
        }
    }

    TextureRgba8 { width: width as u32, height: height as u32, pixels }
}

/// メイドちゃん(魔法少女、独自デザイン、初音ミク風フィギュアを持つ設定は
/// ゲームロジック側の演出——スプライト自体はオリジナル意匠)。
pub const MAID: &[&str] = &[
    "...HHHH...",
    "..HHHHHH..",
    "..SSSSSS..",
    ".SSKKKKSS.",
    "..SSSSSS..",
    ".DDDDDDDD.",
    "DDPPPPPPDD",
    "DDPPPPPPDD",
    ".D......D.",
    ".D......D.",
];

/// 助っ人「風天のトラさん」(寅次郎風キャラのオマージュ、独自デザイン)。
pub const HELPER: &[&str] = &[
    "...KKKK...",
    "..KKKKKK..",
    "..SSSSSS..",
    ".SSKKKKSS.",
    "..SSSSSS..",
    ".TTTTTTTT.",
    "TTAATTAATT",
    "TTAATTAATT",
    ".T......T.",
    ".T......T.",
];

/// 魔女(大悪魔、独自デザイン)。
pub const WITCH: &[&str] = &[
    "....VVVV....",
    "...VVVVVV...",
    "..VVSSSSVV..",
    "..V SKKKS V.",
    "..VVSSSSVV..",
    ".WWWWWWWWWW.",
    "WWWPPPPPPWWW",
    "WWWPPPPPPWWW",
    "..W......W..",
    ".WW......WW.",
];

/// 雑魚敵(味方化前、独自デザイン)。
pub const ENEMY: &[&str] = &[
    "....EEEE....",
    "..EEEEEEEE..",
    ".EEFFFFFFEE.",
    "EEFF.EE.FFEE",
    "EEFF....FFEE",
    ".EEFFFFFFEE.",
    "..EEEEEEEE..",
    "....E..E....",
];

pub fn maid_palette(c: char) -> Rgba8 {
    match c {
        'H' => Rgba8 { r: 250, g: 210, b: 220, a: 255 },
        'S' => Rgba8 { r: 250, g: 225, b: 200, a: 255 },
        'K' => Rgba8 { r: 60, g: 40, b: 40, a: 255 },
        'D' => Rgba8 { r: 240, g: 150, b: 190, a: 255 },
        'P' => Rgba8 { r: 255, g: 255, b: 255, a: 255 },
        _ => Rgba8 { r: 20, g: 10, b: 20, a: 255 },
    }
}

pub fn helper_palette(c: char) -> Rgba8 {
    match c {
        'K' => Rgba8 { r: 60, g: 40, b: 30, a: 255 },
        'S' => Rgba8 { r: 235, g: 190, b: 150, a: 255 },
        'T' => Rgba8 { r: 200, g: 120, b: 50, a: 255 },
        'A' => Rgba8 { r: 240, g: 200, b: 90, a: 255 },
        _ => Rgba8 { r: 20, g: 10, b: 20, a: 255 },
    }
}

pub fn witch_palette(c: char) -> Rgba8 {
    match c {
        'V' => Rgba8 { r: 60, g: 20, b: 90, a: 255 },
        'S' => Rgba8 { r: 220, g: 200, b: 200, a: 255 },
        'K' => Rgba8 { r: 30, g: 10, b: 10, a: 255 },
        'W' => Rgba8 { r: 120, g: 40, b: 160, a: 255 },
        'P' => Rgba8 { r: 160, g: 60, b: 200, a: 255 },
        _ => Rgba8 { r: 20, g: 10, b: 20, a: 255 },
    }
}

pub fn enemy_notally_palette(c: char) -> Rgba8 {
    match c {
        'E' => Rgba8 { r: 110, g: 40, b: 130, a: 255 },
        'F' => Rgba8 { r: 150, g: 50, b: 170, a: 255 },
        _ => Rgba8 { r: 20, g: 10, b: 20, a: 255 },
    }
}

pub fn enemy_ally_palette(c: char) -> Rgba8 {
    match c {
        'E' => Rgba8 { r: 50, g: 150, b: 190, a: 255 },
        'F' => Rgba8 { r: 90, g: 200, b: 230, a: 255 },
        _ => Rgba8 { r: 20, g: 10, b: 20, a: 255 },
    }
}

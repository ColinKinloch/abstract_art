//! Art may be a little hard to understand, but it's easy to see that this fight will be no problem.
//! # なぞのゲージュツ
// Nazo no gējutsu
//!
//! Using vulkan

extern crate vulkano;

pub mod rom_util;

pub mod layer;
pub mod battle_group;

use std::ops::Range;

pub const BG_BANK: &'static [u8] = include_bytes!("bgbank.bin");

pub mod draw_vs {
    include!{concat!(env!("OUT_DIR"), "/shaders/src/art/draw.glslv")}
}
pub mod aa_fs {
    include!{concat!(env!("OUT_DIR"), "/shaders/src/art/abstract_art.glslf")}
}
pub mod compose_fs {
    include!{concat!(env!("OUT_DIR"), "/shaders/src/art/compose.glslf")}
}

pub const LAYER_MAX: u16 = 326;
pub const BATTLE_GROUP_MAX: u16 = 484;

pub const PALETTE_MAX: u16 = 0x10;

// TODO: Retype these to u16?
pub const MAP_WIDTH: u32 = 256;
pub const MAP_HEIGHT: u32 = 256;

// TODO: Implement wipe?

// TODO: Rename these?
const START: usize = 0xA0200;
const GRAPHICS_PTR_RANGE: Range<usize> = 0xAD9A1..0xADB3D;
const ARRANGEMENT_PTR_RANGE: Range<usize> = 0xADB3D..0xADCD9;
const PALETTE_PTR_RANGE: Range<usize> = 0xADCD9..0xADEA1;
const RENDERING_DATA_RANGE: Range<usize> = 0xADEA1..0xAF458;
const TRANSLATION_RANGE: Range<usize> = 0xAF458..0xAF908;
const DISTORTION_RANGE: Range<usize> = 0xAF908..0xB01FF;
const BATTLE_GROUP_RANGE: Range<usize> = 0xBDA9A..0xBE22A;

pub const PRIMARY_DATA: u32 = START as u32;
pub const GRAPHICS_PTR_MAP: Range<usize> = (GRAPHICS_PTR_RANGE.start - START) .. (GRAPHICS_PTR_RANGE.end - START);
pub const ARRANGEMENT_PTR_MAP: Range<usize> = (ARRANGEMENT_PTR_RANGE.start - START) .. (ARRANGEMENT_PTR_RANGE.end - START);
pub const PALETTE_PTR_MAP: Range<usize> = (PALETTE_PTR_RANGE.start - START) .. (PALETTE_PTR_RANGE.end - START);
pub const RENDERING_DATA_MAP: Range<usize> = (RENDERING_DATA_RANGE.start - START) .. (RENDERING_DATA_RANGE.end - START);
pub const TRANSLATION_MAP: Range<usize> = (TRANSLATION_RANGE.start - START) .. (TRANSLATION_RANGE.end - START);
pub const DISTORTION_MAP: Range<usize> = (DISTORTION_RANGE.start - START) .. (DISTORTION_RANGE.end - START);
pub const BATTLE_GROUP_MAP: Range<usize> = (BATTLE_GROUP_RANGE.start - START) .. (BATTLE_GROUP_RANGE.end - START);

pub const TILE_MAX: u16 = 0x3740;
pub const ARRANGE_MAX: u16 = 0x800;

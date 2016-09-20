//! Okay maybe this

use super::rom_util;
use super::{BG_BANK, PRIMARY_DATA,
            TILE_MAX, PALETTE_MAX,
            ARRANGE_MAX, RENDERING_DATA_MAP, DISTORTION_MAP, TRANSLATION_MAP,
            GRAPHICS_PTR_MAP, ARRANGEMENT_PTR_MAP, PALETTE_PTR_MAP, LAYER_MAX};

use std::mem;
use std::fmt;

use std::ops::Range;

/// Represents raw layer data from ROM
#[derive(Debug, Clone)]
#[repr(C,packed)]
struct LayerData {
    map_index: u8,
    palette_index: u8,
    bpp: u8,
    cycle_type: u8,
    cycles: [(u8, u8); 2],
    cycle_speed: u8,
    translations: [u8; 4],
    distortions: [u8; 4]
}

#[derive(Debug, Clone)]
#[repr(C,packed)]
pub struct Translation {
    pub duration: u16,
    pub velocity: (i16, i16),
    pub acceleration: (i16, i16),
}

// TODO: Vertical Compression
#[derive(Debug, Clone)]
#[repr(u8)]
pub enum DistortionType {
    None = 0,
    Horizontal = 1,
    Interlaced = 2,
    Vertical = 3,
    Shear = 4,
    Compression = 5, // ???
}

#[derive(Debug, Clone)]
#[repr(C,packed)]
pub struct Distortion {
    pub duration: u16,
    pub style: DistortionType,
    pub frequency: u16,
    pub amplitude: u16,
    pub unknown_hh: i8,
    pub compression: i16,
    pub frequency_delta: i16,
    pub amplitude_delta: i16,
    pub speed: i8,
    pub compression_delta: i16,
}

#[derive(Debug, Clone)]
#[repr(u8)]
pub enum CycleType {
    None = 0,
    Rotate1 = 1,
    Rotate2 = 2,
    Triange = 3,
}

pub struct Layer {
    pub map: [u8; 256 * 256],
    pub palettes: [[u16; PALETTE_MAX as usize]; 8],
    pub translations: [Option<Translation>; 4],
    pub distortions: [Option<Distortion>; 4],
    pub style: CycleType,
    pub speed: u8,
    pub cycles: [Range<u8>; 2],
}

// TODO: Tidy this whole thing
impl fmt::Debug for Layer {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    use std::io::Write;
    use std::str;
    let map = {
      const SIZE: usize = 32;
      const DIFF: usize = 256 / SIZE;
      let mut map = [0.; SIZE * SIZE];
      let shades = [" ", "░", "▒", "▓", "█"];
      let mut high = 0.;
      // Average
      for (i, v) in self.map.iter().enumerate() {
        let x = i % 256 / DIFF;
        let y = i / 256 / DIFF;
        map[y * SIZE + x] += (*v as f32 / (16. * (DIFF * DIFF) as f32)) as f32;
        if map[y * SIZE + x] > high { high = map[y * SIZE + x]; }
      }
      let mut range = 255. .. 0.;
      // Normalise
      for v in map.iter() {
        if *v > range.end { range.end = *v; }
        if *v < range.start { range.start = *v; }
      }
      let depth = range.end - range.start;
      // Select Shade
      let mut map_string = Vec::new();
      for (i, v) in map.iter().enumerate() {
        if i % SIZE == 0 {
          write!(&mut map_string, "\n").unwrap();
        }
        if depth == 0. {
          write!(&mut map_string, "{}", shades[0]).unwrap();
        } else {
          write!(&mut map_string, "{}", shades[((shades.len() - 1) as f32 * ((v - range.start) / depth)) as usize]).unwrap();
        }
  //      println!("{}", v);
      }
      String::from_utf8(map_string).unwrap()
    };
    // TODO: Palette rendering?
    write!(f, "Layer {{ map: {}, \npalettes: [], \ntranslations: {:?}, \ndistortions: {:?}, \nstyle: {:?}, speed: {:?}, cycles: {:?} }}", map, self.translations, self.distortions, self.style, self.speed, self.cycles)
  }
}

impl Layer {
    // TODO: Break function up?
    /// This function puts the fun back in procedure
    pub fn for_index(index: u16) -> Result<Layer, String> {
        if index > LAYER_MAX {
            return Err("Trying to read non existent image".to_string());
        }

        // TODO: Are these actualy safe?
        // TODO: Test all for valid data (Large number of similar bgs)
        let rendering_data = unsafe { mem::transmute::<&[u8], &[LayerData]>(&BG_BANK[RENDERING_DATA_MAP]) };
        
        let translation_data = unsafe { mem::transmute::<&[u8], &[Translation]>(&BG_BANK[TRANSLATION_MAP]) };
        let distortion_data = unsafe { mem::transmute::<&[u8], &[Distortion]>(&BG_BANK[DISTORTION_MAP]) };

        let graphics_ptr_data = unsafe { mem::transmute::<&[u8], &[i32]>(&BG_BANK[GRAPHICS_PTR_MAP]) };
        let arrangement_ptr_data = unsafe { mem::transmute::<&[u8], &[i32]>(&BG_BANK[ARRANGEMENT_PTR_MAP]) };
        let palette_ptr_data = unsafe { mem::transmute::<&[u8], &[i32]>(&BG_BANK[PALETTE_PTR_MAP]) };

        let ref layer_data = rendering_data[index as usize];

        let mut layer = Layer {
            map: [0; 256 * 256],
            palettes: Default::default(),
            style: match layer_data.cycle_type {
                1 => CycleType::Rotate1,
                2 => CycleType::Rotate2,
                3 => CycleType::Triange,
                _ => CycleType::None,
            },
            speed: layer_data.cycle_speed,
            translations: Default::default(),
            distortions: Default::default(),
            cycles: [0..0, 0..0]
        };
        
        for (o, i) in layer.cycles.iter_mut().zip(layer_data.cycles.iter()) {
            *o = i.0..i.1;
        }

        for (trans, index) in layer.translations.iter_mut().zip(layer_data.translations.iter()) {
            if *index > 0 {
                *trans = Some(translation_data[*index as usize].clone());
            }
            
        }
        for (dist, index) in layer.distortions.iter_mut().zip(layer_data.distortions.iter()) {
            if *index > 0 {
                *dist = Some(distortion_data[*index as usize].clone());
            }
            
        }

        let p_palette_data = rom_util::from_snes(palette_ptr_data[layer_data.palette_index as usize]) -
                             PRIMARY_DATA as i32;
        let palette_data = unsafe { mem::transmute::<&[u8], &[u16]>(&BG_BANK[p_palette_data as usize..(p_palette_data as u32 + PALETTE_MAX as u32 * 8) as usize]) };

        // Get palettes from 0B5G5R5
        // Why 8 rows?
        let mut c = palette_data.iter();
        for palette in layer.palettes.iter_mut() {
            for colour in palette.iter_mut().take((1 << layer_data.bpp) as usize) {
        // From A1B5G5R5 to B5G5R5A1
        *colour = *c.next().unwrap() << 1;

        // From A1B5G5R5 to R8G8B8A8
        /*let b = ((c >> 10) & 0x1F) as u8;
        let g = ((c >> 5) & 0x1F) as u8;
        let r = (c & 0x1F) as u8;

        // scale to RGB888 values
        let s = 255/63;
        *colour = (r * s, g * s, b * s);*/
      }
        }

        let p_tile_data = rom_util::from_snes(graphics_ptr_data[layer_data.map_index as usize]) as u32 -
                          PRIMARY_DATA;
        let tile_data = rom_util::decompress(BG_BANK, p_tile_data, TILE_MAX as u32);
        let td = tile_data.unwrap();

        let p_arrange_data = rom_util::from_snes(arrangement_ptr_data[layer_data.map_index as usize]) as u32 -
                             PRIMARY_DATA;
        let arrange_data = rom_util::decompress(BG_BANK, p_arrange_data, ARRANGE_MAX as u32);

        // Make tiles
        let n = td.len() as u16 / (8 * layer_data.bpp as u16);

        let mut tiles = Vec::new();

        for i in 0..n {
            let o = i as u16 * 8 * layer_data.bpp as u16;
            let mut tile = [[0; 8]; 8];
            for y in 0..8 {
                for x in 0..8 {
                    let mut c = 0;
                    for bp in 0..layer_data.bpp {
                        c += ((td[(o + x * 2 + ((bp / 2) * 16 + (bp & 1)) as u16) as usize] &
                               (1 << 7 - y)) >> 7 - y) << bp;
                    }
                    tile[y as usize][x as usize] = c as u8;
                }
            }
            tiles.push(tile);
        }


        // Make images
        let arrange = arrange_data.unwrap();

        for y in 0..32 {
            for x in 0..32 {
                let n = y * 32 + x;

                let block = {
                    let b = arrange[n * 2] as u16;
                    b + ((arrange[n * 2 + 1] as u16) << 8)
                };

                let tile = block & 0x3FF;
                let vflip = (block & 0x8000) != 0;
                let hflip = (block & 0x4000) != 0;
                // let subpal = (block >> 10) & 7;

                // HACK HERE?

                for i in 0..8 {
                    for j in 0..8 {
                        let px = {
                            if hflip {
                                (x * 8) + 7 - i
                            } else {
                                (x * 8) + i
                            }
                        };
                        let py = {
                            if vflip {
                                (y * 8) + 7 - j
                            } else {
                                (y * 8) + j
                            }
                        };
                        let stride = 256;
                        let colour = tiles[tile as usize][i][j];
                        let pos = px + py * stride;
                        layer.map[pos] = colour;
                    }
                }
            }
        }

        Ok(layer)
    }
}

impl Default for Layer {
    fn default() -> Layer {
        Layer {
            map: [0; 256 * 256],
            palettes: [[0; PALETTE_MAX as usize]; 8],
            style: CycleType::None,
            translations: [None, None, None, None],
            distortions: [None, None, None, None],
            cycles: [0..0, 0..0],
            speed: 0,
        }
    }
}

#[test]
fn load_all_layers() {
    for i in 0..super::LAYER_MAX {
        Layer::for_index(i).unwrap();
    }
}

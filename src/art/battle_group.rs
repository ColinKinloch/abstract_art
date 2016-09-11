use super::layer::Layer;

use super::{BG_BANK, BATTLE_GROUP_MAP};

use std::mem::transmute;

pub struct BattleGroup {
    pub layers: [Layer; 2],
}
impl BattleGroup {
    pub fn for_index(index: u16) -> Result<BattleGroup, String> {
        let battle_group_data = unsafe { transmute::<&[u8], &[u16]>(&BG_BANK[BATTLE_GROUP_MAP]) };
        let i = index * 2;
        let layers = [Layer::for_index(battle_group_data[i as usize]).unwrap(),
                      Layer::for_index(battle_group_data[1 + i as usize]).unwrap()];
        Ok(BattleGroup { layers: layers })
    }
}
impl Default for BattleGroup {
    fn default() -> BattleGroup {
        BattleGroup { layers: Default::default() }
    }
}

#[test]
fn load_all_battle_groups() {
    for i in 0..super::BATTLE_GROUP_MAX {
        BattleGroup::for_index(i).unwrap();
    }
}

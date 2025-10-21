use crate::Simulator;
use crate::damage_type::DamageType;

pub struct Hit {
    pub r#type: DamageType,
    pub dmg: i32,
    pub is_crit: bool,
}

impl Hit {
    pub fn new(
        attack_dmg: i32,
        dmg_increase: i32,
        r#type: DamageType,
        target_res: i32,
        is_crit: bool,
    ) -> Hit {
        let mut dmg = attack_dmg as f32;

        dmg *= if is_crit {
            Simulator::crit_multiplier(dmg_increase, target_res)
        } else {
            Simulator::critless_multiplier(dmg_increase, target_res)
        };
        Hit {
            r#type,
            dmg: dmg.round() as i32,
            is_crit,
        }
    }

    pub fn averaged(
        attack_dmg: i32,
        dmg_increase: i32,
        critical_strike: i32,
        r#type: DamageType,
        target_res: i32,
    ) -> Hit {
        let mut dmg = attack_dmg as f32;

        dmg *= Simulator::average_multiplier(dmg_increase, critical_strike, target_res);
        Hit {
            r#type,
            dmg: dmg.round() as i32,
            is_crit: true,
        }
    }
}



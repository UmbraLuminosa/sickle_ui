use std::f32::consts::PI;

use bevy::reflect::Reflect;
use serde::{Deserialize, Serialize};

const C1_F32: f32 = 1.70158;
const C2_F32: f32 = C1_F32 * 1.525;
const C3_F32: f32 = C1_F32 + 1.;
const C4_F32: f32 = (2. * PI) / 3.;
const C5_F32: f32 = (2. * PI) / 4.5;
const N1_F32: f32 = 7.5625;
const D1_F32: f32 = 2.75;

// const PI_64: f64 = std::f64::consts::PI;
// const C1_F64: f64 = 1.70158;
// const C2_F64: f64 = C1_F64 * 1.525;
// const C3_F64: f64 = C1_F64 + 1.;
// const C4_F64: f64 = (2. * PI_64) / 3.;
// const C5_F64: f64 = (2. * PI_64) / 4.5;
// const N1_F64: f64 = 7.5625;
// const D1_F64: f64 = 2.75;

#[derive(Default, Copy, Clone, Debug, Hash, PartialEq, Eq, Reflect, Serialize, Deserialize)]
pub enum Ease {
    #[default]
    Linear,
    InSine,
    OutSine,
    InOutSine,
    InQuad,
    OutQuad,
    InOutQuad,
    InCubic,
    OutCubic,
    InOutCubic,
    InQuart,
    OutQuart,
    InOutQuart,
    InQuint,
    OutQuint,
    InOutQuint,
    InExpo,
    OutExpo,
    InOutExpo,
    InCirc,
    OutCirc,
    InOutCirc,
    InBack,
    OutBack,
    InOutBack,
    InElastic,
    OutElastic,
    InOutElastic,
    InBounce,
    OutBounce,
    InOutBounce,
}

pub trait ValueEasing {
    fn ease(&self, ease_type: Ease) -> Self;
}

impl ValueEasing for f32 {
    fn ease(&self, ease_type: Ease) -> Self {
        let x = self.clamp(0., 1.);

        match ease_type {
            Ease::Linear => x,
            Ease::InSine => 1. - ((x * PI) / 2.).cos(),
            Ease::OutSine => ((x * PI) / 2.).sin(),
            Ease::InOutSine => -((PI * x).cos() - 1.) / 2.,
            Ease::InQuad => x * x,
            Ease::OutQuad => 1. - (1. - x) * (1. - x),
            Ease::InOutQuad => {
                if x < 0.5 {
                    2. * x * x
                } else {
                    1. - (-2. * x + 2.).powi(2) / 2.
                }
            }
            Ease::InCubic => x * x * x,
            Ease::OutCubic => 1. - (1. - x).powi(3),
            Ease::InOutCubic => {
                if x < 0.5 {
                    4. * x * x * x
                } else {
                    1. - (-2. * x + 2.).powi(3) / 2.
                }
            }
            Ease::InQuart => x * x * x * x,
            Ease::OutQuart => 1. - (1. - x).powi(4),
            Ease::InOutQuart => {
                if x < 0.5 {
                    8. * x * x * x * x
                } else {
                    1. - (-2. * x + 2.).powi(4) / 2.
                }
            }
            Ease::InQuint => x * x * x * x * x,
            Ease::OutQuint => 1. - (1. - x).powi(5),
            Ease::InOutQuint => {
                if x < 0.5 {
                    16. * x * x * x * x * x
                } else {
                    1. - (-2. * x + 2.).powi(5) / 2.
                }
            }
            Ease::InExpo => {
                if x == 0. {
                    0.
                } else {
                    (2f32).powf(10. * x - 10.)
                }
            }
            Ease::OutExpo => {
                if x == 1. {
                    1.
                } else {
                    1. - 2f32.powf(-10. * x)
                }
            }
            Ease::InOutExpo => {
                if x == 0. {
                    0.
                } else {
                    if x == 1. {
                        1.
                    } else {
                        if x < 0.5 {
                            2f32.powf(20. * x - 10.) / 2.
                        } else {
                            (2. - 2f32.powf(-20. * x + 10.)) / 2.
                        }
                    }
                }
            }
            Ease::InCirc => 1. - (1. - x.powi(2)).sqrt(),
            Ease::OutCirc => (1. - (x - 1.).powi(2)).sqrt(),
            Ease::InOutCirc => {
                if x < 0.5 {
                    (1. - (1. - (2. * x).powi(2)).sqrt()) / 2.
                } else {
                    ((1. - (-2. * x + 2.).powi(2)).sqrt() + 1.) / 2.
                }
            }
            Ease::InBack => C3_F32 * x * x * x - C1_F32 * x * x,
            Ease::OutBack => 1. + C3_F32 * (x - 1.).powi(3) + C1_F32 * (x - 1.).powi(2),
            Ease::InOutBack => {
                if x < 0.5 {
                    ((2. * x).powi(2) * ((C2_F32 + 1.) * 2. * x - C2_F32)) / 2.
                } else {
                    ((2. * x - 2.).powi(2) * ((C2_F32 + 1.) * (x * 2. - 2.) + C2_F32) + 2.) / 2.
                }
            }
            Ease::InElastic => {
                if x == 0. {
                    0.
                } else {
                    if x == 1. {
                        1.
                    } else {
                        -2f32.powf(10. * x - 10.) * ((x * 10. - 10.75) * C4_F32).sin()
                    }
                }
            }
            Ease::OutElastic => {
                if x == 0. {
                    0.
                } else {
                    if x == 1. {
                        1.
                    } else {
                        2f32.powf(-10. * x) * ((x * 10. - 0.75) * C4_F32).sin() + 1.
                    }
                }
            }
            Ease::InOutElastic => {
                if x == 0. {
                    0.
                } else {
                    if x == 1. {
                        1.
                    } else {
                        if x < 0.5 {
                            -(2f32.powf(20. * x - 10.) * ((20. * x - 11.125) * C5_F32).sin()) / 2.
                        } else {
                            (2f32.powf(-20. * x + 10.) * ((20. * x - 11.125) * C5_F32).sin()) / 2.
                                + 1.
                        }
                    }
                }
            }
            Ease::InBounce => 1. - (1. - x).ease(Ease::InOutBounce),
            Ease::OutBounce => {
                if x < 1. / D1_F32 {
                    N1_F32 * x * x
                } else if x < 2. / D1_F32 {
                    let x = x - 1.5 / D1_F32;
                    N1_F32 * x * x + 0.75
                } else if x < 2.5 / D1_F32 {
                    let x = x - 2.25 / D1_F32;
                    N1_F32 * x * x + 0.9375
                } else {
                    let x = x - 2.625 / D1_F32;
                    N1_F32 * x * x + 0.984375
                }
            }
            Ease::InOutBounce => {
                if x < 0.5 {
                    (1. - (1. - 2. * x).ease(Ease::OutBounce)) / 2.
                } else {
                    (1. + (2. * x - 1.).ease(Ease::OutBounce)) / 2.
                }
            }
        }
    }
}

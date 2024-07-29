use bevy_color::{Color, Mix};
use bevy_ui::{BorderRadius, Outline, UiRect, Val};

pub trait Lerp {
    fn lerp(&self, to: Self, t: f32) -> Self;
}

pub trait Lerp64 {
    fn lerp_64(&self, to: Self, t: f64) -> Self;
}

impl Lerp for f32 {
    fn lerp(&self, to: Self, t: f32) -> Self {
        self + ((to - self) * t)
    }
}

impl Lerp for f64 {
    fn lerp(&self, to: Self, t: f32) -> Self {
        self + ((to - self) * t as f64)
    }
}

impl Lerp64 for f32 {
    fn lerp_64(&self, to: Self, t: f64) -> Self {
        self + ((to - self) * t as f32)
    }
}

impl Lerp64 for f64 {
    fn lerp_64(&self, to: Self, t: f64) -> Self {
        self + ((to - self) * t)
    }
}

impl Lerp for usize {
    /// NOTE: This will try to convert the `usize` into `f64` for calculation. Falls back to 0.
    fn lerp(&self, to: Self, t: f32) -> Self {
        let a = f64::try_from(*self as u32).unwrap_or_default();
        let b = f64::try_from(to as u32).unwrap_or_default();

        a.lerp(b, t).round() as usize
    }
}

impl Lerp for Color {
    fn lerp(&self, to: Self, t: f32) -> Self {
        self.mix(&to, t)
    }
}

// TODO: Create a derive macro for these types?
impl Lerp for BorderRadius {
    fn lerp(&self, to: Self, t: f32) -> Self {
        Self {
            top_left: self.top_left.lerp(to.top_left, t),
            top_right: self.top_right.lerp(to.top_right, t),
            bottom_left: self.bottom_left.lerp(to.bottom_left, t),
            bottom_right: self.bottom_right.lerp(to.bottom_right, t),
        }
    }
}

impl Lerp for Outline {
    fn lerp(&self, to: Self, t: f32) -> Self {
        Self {
            width: self.width.lerp(to.width, t),
            offset: self.offset.lerp(to.offset, t),
            color: self.color.lerp(to.color, t),
        }
    }
}

impl Lerp for Val {
    fn lerp(&self, to: Self, t: f32) -> Self {
        // We can only LERP between values with the same scale
        match self {
            Val::Auto => self.clone(),
            Val::Px(value) => {
                if let Val::Px(other) = to {
                    Self::Px(value.lerp(other, t))
                } else {
                    self.clone()
                }
            }
            Val::Percent(value) => {
                if let Val::Percent(other) = to {
                    Self::Percent(value.lerp(other, t))
                } else {
                    self.clone()
                }
            }
            Val::Vw(value) => {
                if let Val::Vw(other) = to {
                    Self::Vw(value.lerp(other, t))
                } else {
                    self.clone()
                }
            }
            Val::Vh(value) => {
                if let Val::Vh(other) = to {
                    Self::Vh(value.lerp(other, t))
                } else {
                    self.clone()
                }
            }
            Val::VMin(value) => {
                if let Val::VMin(other) = to {
                    Self::VMin(value.lerp(other, t))
                } else {
                    self.clone()
                }
            }
            Val::VMax(value) => {
                if let Val::VMax(other) = to {
                    Self::VMax(value.lerp(other, t))
                } else {
                    self.clone()
                }
            }
        }
    }
}

impl Lerp for UiRect {
    fn lerp(&self, to: Self, t: f32) -> Self {
        Self::new(
            self.left.lerp(to.left, t),
            self.right.lerp(to.right, t),
            self.top.lerp(to.top, t),
            self.bottom.lerp(to.bottom, t),
        )
    }
}

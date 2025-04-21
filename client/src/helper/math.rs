pub trait RoundTo {
    /// Rounds `self` to `decimals` places.
    fn round_to(self, decimals: u32) -> f32;
}

impl RoundTo for f32 {
    #[inline]
    fn round_to(self, decimals: u32) -> f32 {
        let factor = 10f32.powi(decimals as i32);
        (self * factor).round() / factor
    }
}
pub struct ContemptParams {
    wdl_rescale_ratio: f32,
    wdl_rescale_diff: f32
}

impl ContemptParams {
    pub fn new(wdl_rescale_ratio: f32, wdl_rescale_diff: f32) -> Self {
        Self { 
            wdl_rescale_ratio,
            wdl_rescale_diff
        }
    }

    pub fn wdl_rescale_ratio(&self) -> f32 {
        self.wdl_rescale_ratio
    }

    pub fn wdl_rescale_diff(&self) -> f32 {
        self.wdl_rescale_diff
    }
}
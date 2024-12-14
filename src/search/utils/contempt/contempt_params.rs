use crate::EngineOptions;

pub struct ContemptParams {
    wdl_rescale_ratio: f32,
    wdl_rescale_diff: f32
}

impl ContemptParams {
    pub fn calculate_params(options: &EngineOptions) -> Self {

        let mut draw_rate_target = options.draw_rate_target();
        if draw_rate_target > 0.0 && draw_rate_target < 0.001 {
            draw_rate_target = 0.001;
        }

        let draw_rate_reference = options.draw_rate_reference();
        let scale_reference = 1.0 / ((1.0 + draw_rate_reference) / (1.0 - draw_rate_reference)).ln();

        let scale_target = if draw_rate_target == 0.0 {
            scale_reference
        } else {
            1.0 / ((1.0 + draw_rate_target) /
                               (1.0 - draw_rate_target)).ln()
        };

        let wdl_rescale_ratio = scale_target / scale_reference;
        let wdl_rescale_diff = scale_target / (scale_reference * scale_reference) / (1.0 / ((0.5 * (1.0 - options.book_exit_bias()) / scale_target).cosh()).powi(2) + 1.0 / ((0.5 * (1.0 + options.book_exit_bias()) / scale_target).cosh()).powi(2) ) * (10.0_f32).ln() / 200.0 * options.contempt().clamp(-options.contempt_max(), options.contempt_max()) * options.contempt_attenuation();

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
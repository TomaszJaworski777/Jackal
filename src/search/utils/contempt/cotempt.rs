use super::ContemptParams;
use crate::EngineOptions;

pub struct Contempt;
impl Contempt {
    pub fn wdl_rescale<const US: bool>(
        v: &mut f32,
        d: &mut f32,
        options: &EngineOptions,
        contempt_parms: &ContemptParams,
    ) {
        let sign = if US { 1.0 } else { -1.0 };

        let w = (1.0 + *v - *d) / 2.0;
        let l = (1.0 - *v - *d) / 2.0;

        const EPS: f32 = 0.0001;
        if w > EPS && *d > EPS && l > EPS && w < (1.0 - EPS) && *d < (1.0 - EPS) && l < (1.0 - EPS)
        {
            let a = (1.0 / l - 1.0).ln();
            let b = (1.0 / w - 1.0).ln();
            let mut s = 2.0 / (a + b);
            s = s.min(options.max_reasonable_s());

            let mu = (a - b) / (a + b);
            let s_new = s * contempt_parms.wdl_rescale_ratio();
            let mu_new = mu + sign * s * s * contempt_parms.wdl_rescale_diff();

            let w_new = fast_logistic((-1.0 + mu_new) / s_new);
            let l_new = fast_logistic((-1.0 - mu_new) / s_new);

            *v = w_new - l_new;
            *d = (1.0 - w_new - l_new).max(0.0);
        }
    }
}

fn fast_logistic(x: f32) -> f32 {
    1.0 / (1.0 + (-x).exp())
}

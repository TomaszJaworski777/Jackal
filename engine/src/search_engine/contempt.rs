use std::mem::swap;

use crate::search_engine::engine_options::EngineOptions;

#[derive(Debug, Clone, Copy)]
pub struct Contempt {
    rescale_ratio: f64,
    rescale_diff: f64
}

impl Contempt {
    pub fn init(options: &EngineOptions) -> Self {
        let scale_reference = 1.0 / ((1.0 + options.draw_rate_ref()) / (1.0 - options.draw_rate_ref())).ln();

        let scale_target = if options.draw_rate_target() == 0.0 {
            scale_reference
        } else {
            1.0 / ((1.0 + options.draw_rate_target()) / (1.0 - options.draw_rate_target())).ln()
        };

        let rescale_ratio = scale_target / scale_reference;
        let rescale_diff = scale_target
            / (scale_reference * scale_reference)
            / (1.0 / ((0.5 * (1.0 - options.book_exit_bias()) / scale_target).cosh()).powi(2)
                + 1.0 / ((0.5 * (1.0 + options.book_exit_bias()) / scale_target).cosh()).powi(2))
            * (10.0_f64).ln()
            / 200.0
            * options.contempt() as f64
            * options.contempt_att();

        Self { rescale_ratio, rescale_diff }
    }

    pub fn rescale(&self, win_lose_delta: &mut f64, draw_chance: &mut f64, sign: f64, invert: bool, options: &EngineOptions) {
        let mut diff = self.rescale_diff;
        let mut ratio = self.rescale_ratio;

        if invert {
            diff = -diff;
            ratio = 1.0 / ratio;
        }

        let win_chance = (1.0 + *win_lose_delta - *draw_chance) / 2.0;
        let lose_chance = (1.0 - *win_lose_delta - *draw_chance) / 2.0;

        const EPS: f64 = 0.0001;
        if win_chance > EPS && *draw_chance > EPS && lose_chance > EPS && win_chance < (1.0 - EPS) && *draw_chance < (1.0 - EPS) && lose_chance < (1.0 - EPS)
        {
            let a = (1.0 / lose_chance - 1.0).ln();
            let b = (1.0 / win_chance - 1.0).ln();
            let mut s = 2.0 / (a + b);

            if !invert {
                s = s.min(options.max_reasonable_s());
            }

            let mu = (a - b) / (a + b);
            let mut s_new = s * ratio;

            if invert {
                swap(&mut s, &mut s_new);
                s = s.min(options.max_reasonable_s());
            }

            let mu_new = mu + sign * s * s * diff;

            let w_new = fast_logistic((-1.0 + mu_new) / s_new);
            let l_new = fast_logistic((-1.0 - mu_new) / s_new);

            *win_lose_delta = w_new - l_new;
            *draw_chance = (1.0 - w_new - l_new).max(0.0);
        }
    }
}

fn fast_logistic(x: f64) -> f64 { //TODO: replace with library later
    1.0 / (1.0 + (-x).exp())
}
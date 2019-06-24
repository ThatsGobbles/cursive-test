#[derive(Copy, Clone)]
pub enum Easing {
    Linear,
    BounceOut,
    Oscillate,
    QuadraticEaseIn,
    QuadraticEaseOut,
}

impl Easing {
    pub fn pos(&self, step: usize, max_steps: usize) -> usize {
        let step = step.min(max_steps);

        if max_steps == 0 { 0 }
        else {
            match self {
                &Easing::Linear => step,
                &Easing::BounceOut => {
                    let t = step as f64 / max_steps as f64;

                    let (t_off, a_off) =
                        if t < 1.0 / 2.75 { (0.0, 0.0) }
                        else if t < 2.0 / 2.75 { (1.5 / 2.75, 0.75) }
                        else if t < 2.5 / 2.75 { (2.25 / 2.75, 0.9375) }
                        else { (2.625 / 2.75, 0.984375) }
                    ;

                    let tt = t - t_off;
                    let f = (7.5625 * tt * tt) + a_off;

                    (max_steps as f64 * f).round() as usize
                },
                &Easing::Oscillate => {
                    let t = step as f64 / max_steps as f64;
                    let f = (t + (8.0 * std::f64::consts::PI * t).sin() / 16.0).clamp(0.0, 1.0);

                    (max_steps as f64 * f).round() as usize
                },
                &Easing::QuadraticEaseIn => {
                    let t = step as f64 / max_steps as f64;
                    (max_steps as f64 * t * t).round() as usize
                },
                &Easing::QuadraticEaseOut => {
                    let t = step as f64 / max_steps as f64;
                    let f = (1.0 - t).clamp(0.0, 1.0);
                    (max_steps as f64 * (1.0 - (f * f))).round() as usize
                },
            }
        }
    }
}

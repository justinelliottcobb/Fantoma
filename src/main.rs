mod audio;

use amari_core::{Bivector, Rotor, Vector};
use std::f64::consts::TAU;

/// Create a closure that generates samples for a sine wave using a geometric
/// algebra rotor to rotate a vector in the XY plane.
fn rotor_oscillator(freq: f64, sample_rate: f64, amplitude: f64) -> impl FnMut() -> f32 + Send {
    let e12: Bivector<3, 0, 0> = Bivector::e12();
    let step_rotor = Rotor::<3, 0, 0>::from_bivector(&e12, TAU * freq / sample_rate);
    let mut state: Vector<3, 0, 0> = Vector::e1();
    let mut sample_count: u64 = 0;
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let renorm_interval = sample_rate as u64;

    move || {
        state = step_rotor.apply_to_vector(&state);
        sample_count += 1;

        if sample_count.is_multiple_of(renorm_interval)
            && let Some(n) = state.normalize()
        {
            state = n;
        }

        #[allow(clippy::cast_possible_truncation)]
        let out = (state.mv.vector_component(1) * amplitude) as f32;
        out
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let sr = audio::sample_rate()?;
    println!("sample rate: {sr} Hz");

    let osc = rotor_oscillator(440.0, f64::from(sr), 0.3);
    let _stream = audio::run_audio(osc)?;

    println!("playing 440 Hz rotor sine — press Enter to stop");
    let mut buf = String::new();
    std::io::stdin().read_line(&mut buf)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rotor_oscillator_range() {
        let mut osc = rotor_oscillator(440.0, 48000.0, 1.0);
        for _ in 0..96_000 {
            let s = osc();
            assert!((-1.1..=1.1).contains(&s), "sample {s} out of range");
        }
    }

    #[test]
    fn rotor_oscillator_crosses_zero() {
        let mut osc = rotor_oscillator(440.0, 48000.0, 1.0);
        let mut has_positive = false;
        let mut has_negative = false;
        for _ in 0..480 {
            let s = osc();
            if s > 0.1 {
                has_positive = true;
            }
            if s < -0.1 {
                has_negative = true;
            }
        }
        assert!(has_positive, "oscillator never went positive");
        assert!(has_negative, "oscillator never went negative");
    }

    #[test]
    fn rotor_oscillator_is_send() {
        fn assert_send<T: Send>(_: &T) {}
        let osc = rotor_oscillator(440.0, 48000.0, 1.0);
        assert_send(&osc);
    }
}

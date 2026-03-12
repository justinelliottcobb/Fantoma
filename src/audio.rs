use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{SampleFormat, Stream};
use std::env;

/// Return true if the device name looks like an HDMI output.
fn is_hdmi(dev: &cpal::Device) -> bool {
    dev.name().ok().is_some_and(|n| {
        n.contains("HDMI")
            || n.contains("DEV=3")
            || n.contains("DEV=7")
            || n.contains("DEV=8")
            || n.contains("DEV=9")
    })
}

/// Find a working output device.
///
/// Selection order:
/// 1. `FANTOMA_DEVICE` env var — ALSA name substring match
/// 2. Default device (if it reports a valid config)
/// 3. First non-HDMI device with a valid config
/// 4. Any device with a valid config
fn output_device() -> Result<cpal::Device, Box<dyn std::error::Error>> {
    let host = cpal::default_host();

    // Allow explicit override via env var
    if let Ok(wanted) = env::var("FANTOMA_DEVICE") {
        let devices = host.output_devices()?;
        for dev in devices {
            if let Ok(name) = dev.name()
                && name.contains(&wanted)
                && dev.default_output_config().is_ok()
            {
                eprintln!("audio: using FANTOMA_DEVICE match \"{name}\"");
                return Ok(dev);
            }
        }
        return Err(format!("FANTOMA_DEVICE=\"{wanted}\" matched no output device").into());
    }

    // Try default first
    if let Some(dev) = host.default_output_device()
        && dev.default_output_config().is_ok()
    {
        return Ok(dev);
    }

    // Prefer non-HDMI analog outputs
    let mut fallback = None;
    let devices = host.output_devices()?;
    for dev in devices {
        if dev.default_output_config().is_ok() {
            if !is_hdmi(&dev) {
                if let Ok(name) = dev.name() {
                    eprintln!("audio: using fallback device \"{name}\"");
                }
                return Ok(dev);
            }
            if fallback.is_none() {
                fallback = Some(dev);
            }
        }
    }

    if let Some(dev) = fallback {
        if let Ok(name) = dev.name() {
            eprintln!("audio: using HDMI fallback \"{name}\"");
        }
        return Ok(dev);
    }

    Err("no working output device found".into())
}

/// Query the output device's sample rate.
///
/// # Errors
/// Returns an error if no output device or config is available.
pub fn sample_rate() -> Result<u32, Box<dyn std::error::Error>> {
    let device = output_device()?;
    let config = device.default_output_config()?;
    Ok(config.sample_rate().0)
}

/// Open audio output and feed it samples from `make_sample`.
///
/// Automatically adapts to the device's native sample format (f32, i16, or u16).
/// The returned `Stream` must be kept alive by the caller.
///
/// # Errors
/// Returns an error if no output device is available or the stream cannot be built.
pub fn run_audio(
    make_sample: impl FnMut() -> f32 + Send + 'static,
) -> Result<Stream, Box<dyn std::error::Error>> {
    let device = output_device()?;
    let supported = device.default_output_config()?;
    let sample_format = supported.sample_format();
    let channels = supported.channels() as usize;

    if let Ok(name) = device.name() {
        eprintln!(
            "audio: opening \"{name}\" @ {} Hz, {} ch, {sample_format:?}",
            supported.sample_rate().0,
            channels,
        );
    }

    let stream_config = supported.config();

    match sample_format {
        SampleFormat::F32 => build_stream_f32(&device, &stream_config, channels, make_sample),
        SampleFormat::I16 => build_stream_i16(&device, &stream_config, channels, make_sample),
        SampleFormat::U16 => build_stream_u16(&device, &stream_config, channels, make_sample),
        other => Err(format!("unsupported sample format: {other:?}").into()),
    }
}

fn build_stream_f32(
    device: &cpal::Device,
    config: &cpal::StreamConfig,
    channels: usize,
    mut make_sample: impl FnMut() -> f32 + Send + 'static,
) -> Result<Stream, Box<dyn std::error::Error>> {
    let stream = device.build_output_stream(
        config,
        move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
            for frame in data.chunks_mut(channels) {
                let sample = make_sample();
                for s in frame {
                    *s = sample;
                }
            }
        },
        |err| eprintln!("audio error: {err}"),
        None,
    )?;
    stream.play()?;
    Ok(stream)
}

fn build_stream_i16(
    device: &cpal::Device,
    config: &cpal::StreamConfig,
    channels: usize,
    mut make_sample: impl FnMut() -> f32 + Send + 'static,
) -> Result<Stream, Box<dyn std::error::Error>> {
    let stream = device.build_output_stream(
        config,
        move |data: &mut [i16], _: &cpal::OutputCallbackInfo| {
            for frame in data.chunks_mut(channels) {
                let sample = make_sample();
                #[allow(clippy::cast_possible_truncation)]
                let s16 = (sample * f32::from(i16::MAX)) as i16;
                for s in frame {
                    *s = s16;
                }
            }
        },
        |err| eprintln!("audio error: {err}"),
        None,
    )?;
    stream.play()?;
    Ok(stream)
}

fn build_stream_u16(
    device: &cpal::Device,
    config: &cpal::StreamConfig,
    channels: usize,
    mut make_sample: impl FnMut() -> f32 + Send + 'static,
) -> Result<Stream, Box<dyn std::error::Error>> {
    let stream = device.build_output_stream(
        config,
        move |data: &mut [u16], _: &cpal::OutputCallbackInfo| {
            for frame in data.chunks_mut(channels) {
                let sample = make_sample();
                // Convert [-1, 1] float to [0, 65535] u16
                #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
                let u = (sample.mul_add(0.5, 0.5) * f32::from(u16::MAX)) as u16;
                for s in frame {
                    *s = u;
                }
            }
        },
        |err| eprintln!("audio error: {err}"),
        None,
    )?;
    stream.play()?;
    Ok(stream)
}

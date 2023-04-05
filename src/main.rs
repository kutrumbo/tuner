use cpal::SampleFormat;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use pitch_detection::detector::mcleod::McLeodDetector;
use pitch_detection::detector::PitchDetector;

// audio processing constants
const SIZE: usize = 512;
const PADDING: usize = SIZE / 2;
const POWER_THRESHOLD: f32 = 1.0;
const CLARITY_THRESHOLD: f32 = 0.3;
// pitch constants
const A4: i16 = 440;
const C0: f32 = (A4 as f32) * 0.03716272234383503; // 2 ^ -4.75;
const NOTES: [&str; 12] = ["C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B"];

fn freq_to_pitch(freq: f32) -> String {
    let h = (12_f32 * (freq / C0).log2()).round();
    let octave = (h as i16) / 12;
    let n = (h as i16) % 12;
    return format!("{}-{}", NOTES[n as usize], octave);
}

fn main() {
    let host = cpal::default_host();
    let device = host.default_input_device().expect("no input device available");

    let mut supported_configs_range = device.supported_input_configs()
        .expect("error while querying configs");
    let supported_config = supported_configs_range.next()
        .expect("no supported config?!")
        .with_max_sample_rate();

    let sample_format = supported_config.sample_format();
    let sample_rate = supported_config.sample_rate();

    println!("Using input device: {}", device.name().unwrap());
    println!("Sample format: {}", sample_format);
    println!("Sample rate: {:?}", sample_rate);
    println!("Channels: {:?}", supported_config.channels());

    let config = supported_config.into();
    let err_fn = |err| eprintln!("an error occurred on the output audio stream: {}", err);

    // only including F32 because that is what shows up locally on my laptop
    let stream = match sample_format {
        SampleFormat::F32 => device.build_input_stream(
            &config,
            move |data: &[f32], _| {
                let mut detector = McLeodDetector::<f32>::new(SIZE, PADDING);
                let pitch = &detector.get_pitch(&data, sample_rate.0 as usize, POWER_THRESHOLD, CLARITY_THRESHOLD);

                match pitch {
                    Some(pitch) => {
                        let note = freq_to_pitch(pitch.frequency);
                        println!("Note: {}, Frequency: {}, Clarity: {}", note, pitch.frequency, pitch.clarity);
                    },
                    None => {
                        // no-op
                    }
                }
            },
            err_fn,
            None
        ),
        sample_format => panic!("Unsupported sample format '{sample_format}'")
    }.unwrap();

    stream.play().unwrap();

    loop {
        // empty to keep looping and reading from input
    }
}

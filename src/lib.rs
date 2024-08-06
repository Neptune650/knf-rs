#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use ndarray::Array2;

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

pub fn compute_fbank(samples: &[f32]) -> Array2<f32> {
    let mut result = unsafe { ComputeFbank(samples.as_ptr(), samples.len().try_into().unwrap()) };

    // Extract frames
    let frames = unsafe {
        std::slice::from_raw_parts(
            result.frames,
            (result.num_frames * result.num_bins) as usize,
        )
        .to_vec()
    };

    let frames_array = Array2::from_shape_vec(
        (
            result.num_frames.try_into().unwrap(),
            result.num_bins.try_into().unwrap(),
        ),
        frames,
    ).unwrap();

    unsafe {
        DestroyFbankResult(&mut result as *mut _);
    }

    let mean = frames_array.mean_axis(ndarray::Axis(0)).unwrap();
    let features = frames_array - mean;

    features
}

#[cfg(test)]
mod tests {
    use crate::compute_fbank;
    use std::f32::consts::PI;

    fn generate_sine_wave(sample_rate: usize, duration: usize, frequency: f32) -> Vec<f32> {
        let waveform_size = sample_rate * duration;
        let mut waveform = Vec::with_capacity(waveform_size);

        for i in 0..waveform_size {
            let sample = 0.5 * (2.0 * PI * frequency * i as f32 / sample_rate as f32).sin();
            waveform.push(sample);
        }
        waveform
    }

    #[test]
    fn it_works() {
        let sample_rate = 16000;
        let duration = 1; // 1 second
        let frequency = 440.0; // A4 note

        let waveform = generate_sine_wave(sample_rate, duration, frequency);
        let features = compute_fbank(&waveform);
        println!("features: {:?}", features);
    }
}

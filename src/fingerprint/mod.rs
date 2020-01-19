use std::error::Error;
use rustfft::algorithm::Radix4;
use rustfft::num_complex::Complex;
use rustfft::num_traits::Zero;
use rustfft::FFT;

const FFT_WINDOW_SIZE: usize = 4096; // chunk window size to process by fast forward fourier function
const FREQ_BINS: &[usize] = &[40, 80, 120, 180, 300]; // Frequencies that separates locality for max magnitude to be calculated (one value per frequencies space) 
const FUZZ_FACTOR: usize = 2; // higher the value of this factor, lower the hash entropy, and less bias the algorithm become to the sound noises

/// Helper struct for calculating acoustic fingerprint
pub struct FingerprintHandle {
    /// FFT algorithm
    fft: Radix4<f32>,
}

impl FingerprintHandle {
    pub fn new() -> FingerprintHandle {
        FingerprintHandle {
            fft: Radix4::new(FFT_WINDOW_SIZE, false),
        }
    }

    /// Calculate fingerprint for decoded stream
    ///
    /// This method uses fast forward fourier computation
    /// to process decoded stream input in to 
    /// stream of complex number output, 
    /// then calculates fingerprint hash
    ///
    /// # Arguments:
    /// * decoded_stream - music that is decoded to stream of floats
    ///
    /// # Returns success of fingerprint hash collection, dynamic error otherwise
    /// 
    pub fn calc_fingerprint_collection(&self, decoded_stream: &[f32]) -> Result<Vec<usize>, Box<dyn Error>> {

        let hash_array = decoded_stream
            .chunks_exact(FFT_WINDOW_SIZE)
            .map(|chunk| {
                let mut input: Vec<Complex<f32>> = chunk.iter().map(Complex::from).collect();
                let mut output: Vec<Complex<f32>> = vec![Complex::zero(); FFT_WINDOW_SIZE];
                self.fft.process(&mut input, &mut output);

                calculate_fingerprint_hash(&output)
            })
            .collect();

        Ok(hash_array)
    }
}

/// Find points with max magnitude in each of the bins
fn calculate_fingerprint_hash(arr: &[Complex<f32>]) -> usize {
    let mut high_scores: Vec<f32> = vec![0.0; FREQ_BINS.len()];
    let mut record_points: Vec<usize> = vec![0; FREQ_BINS.len()];

    for bin in FREQ_BINS[0]..=FREQ_BINS[4] {
        let magnitude = arr[bin].re.hypot(arr[bin].im);

        let mut bin_idx = 0;
        while FREQ_BINS[bin_idx] < bin {
            bin_idx += 1;
        }

        if magnitude > high_scores[bin_idx] {
            high_scores[bin_idx] = magnitude;
            record_points[bin_idx] = bin;
        }
    }

    hash(&record_points)
}


/// Hash function with reverse order
fn hash(arr: &[usize]) -> usize {
    (arr[3] - (arr[3] % FUZZ_FACTOR)) * usize::pow(10, 8)
        + (arr[2] - (arr[2] % FUZZ_FACTOR)) * usize::pow(10, 5)
        + (arr[1] - (arr[1] % FUZZ_FACTOR)) * usize::pow(10, 2)
        + (arr[0] - (arr[0] % FUZZ_FACTOR))
}

#[cfg(test)]
mod tests {
    use rand::prelude::*;
    use rustfft::num_traits::Zero;
    #[test]
    fn test_hash() {
        let record_points_0 = vec![45, 100, 140, 235, 300];
        let record_points_1 = vec![45, 100, 145, 235, 300];
        assert_eq!(super::hash(&record_points_0), 23414010044);
        assert_ne!(super::hash(&record_points_0), super::hash(&record_points_1));
    }
    #[test]
    fn test_calculate_fingerprint_hash() {
        let mut rng = rand::thread_rng();
        let mut arr_f32: Vec<f32> = vec![0.0; super::FFT_WINDOW_SIZE];
        arr_f32.iter_mut().for_each(|complex_num| {
            *complex_num =  rng.gen::<f32>() * 10000_f32;
        });
        let arr: Vec<super::Complex<f32>> = arr_f32.iter().map(super::Complex::from).collect();
        let fingerprint = super::calculate_fingerprint_hash(&arr);
        assert_eq!(fingerprint > 10000000001, true);
    }
}
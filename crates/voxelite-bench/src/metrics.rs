//! Linear-interpolated percentiles and aggregation without hiding run spread.
use crate::Result;
use serde::{Deserialize, Serialize};

pub fn percentile(sorted: &[f64], p: f64) -> Result<f64> {
    if sorted.is_empty()
        || !p.is_finite()
        || !(0.0..=1.0).contains(&p)
        || sorted.iter().any(|v| !v.is_finite())
        || sorted.windows(2).any(|v| v[0] > v[1])
    {
        return Err("percentile needs sorted finite samples and p in [0,1]".into());
    }
    let index = p * (sorted.len() - 1) as f64;
    let lo = index.floor() as usize;
    let hi = index.ceil() as usize;
    let weight = index - lo as f64;
    Ok(sorted[lo] * (1.0 - weight) + sorted[hi] * weight)
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Distribution {
    pub p50: f64,
    pub p95: f64,
    pub p99: f64,
    pub max: f64,
    pub mean: f64,
}
impl Distribution {
    pub fn new(samples: &[f64]) -> Result<Self> {
        let mut sorted = samples.to_vec();
        sorted.sort_by(f64::total_cmp);
        Ok(Self {
            p50: percentile(&sorted, 0.5)?,
            p95: percentile(&sorted, 0.95)?,
            p99: percentile(&sorted, 0.99)?,
            max: sorted
                .last()
                .copied()
                .ok_or("metrics require at least one sample")?,
            mean: sorted.iter().map(|v| v / sorted.len() as f64).sum(),
        })
    }
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Spread {
    pub median: f64,
    pub min: f64,
    pub max: f64,
    pub runs: Vec<f64>,
}
impl Spread {
    pub fn new(runs: Vec<f64>) -> Result<Self> {
        let mut sorted = runs.clone();
        sorted.sort_by(f64::total_cmp);
        let median = percentile(&sorted, 0.5)?;
        Ok(Self {
            median,
            min: sorted[0],
            max: sorted
                .last()
                .copied()
                .ok_or("metrics require at least one sample")?,
            runs,
        })
    }
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Metric {
    pub p50: Spread,
    pub p95: Spread,
    pub p99: Spread,
    pub max: Spread,
    pub mean: Spread,
}
impl Metric {
    pub fn from_runs(runs: &[Distribution]) -> Result<Self> {
        let spread = |f: fn(&Distribution) -> f64| Spread::new(runs.iter().map(f).collect());
        Ok(Self {
            p50: spread(|d| d.p50)?,
            p95: spread(|d| d.p95)?,
            p99: spread(|d| d.p99)?,
            max: spread(|d| d.max)?,
            mean: spread(|d| d.mean)?,
        })
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn percentiles_cover_edges_and_reject_invalid_data() {
        assert!(percentile(&[], 0.5).is_err());
        assert!(percentile(&[1.0], f64::NAN).is_err());
        assert!(percentile(&[f64::INFINITY], 0.5).is_err());
        assert!(percentile(&[2.0, 1.0], 0.5).is_err());
        assert_eq!(percentile(&[7.0], 0.99).unwrap(), 7.0);
        assert_eq!(percentile(&[1.0, 3.0], 0.5).unwrap(), 2.0);
        assert_eq!(percentile(&[0.0, 100.0], 0.95).unwrap(), 95.0);
        assert_eq!(percentile(&[0.0, 100.0], 0.0).unwrap(), 0.0);
        assert_eq!(percentile(&[0.0, 100.0], 1.0).unwrap(), 100.0);
        let spread = Spread::new(vec![8.0, 1.0, 3.0]).unwrap();
        assert_eq!((spread.median, spread.min, spread.max), (3.0, 1.0, 8.0));
        assert_eq!(spread.runs, vec![8.0, 1.0, 3.0]);
    }
}

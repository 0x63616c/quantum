//! Exact integer primitive occupancy and supercover lines.
use super::Work;
use crate::*;
use std::collections::BTreeSet;

pub(super) fn shape_cells(shape: &Shape, work: &mut Work) -> Result<Vec<Pos>> {
    if let Shape::Line { from, to } = shape {
        return line_cells(*from, *to, work);
    }
    let (region, hollow) = match shape {
        Shape::Box { region, hollow } => {
            region.validate()?;
            (*region, *hollow)
        }
        Shape::Ellipsoid {
            center2,
            radii2,
            hollow,
        }
        | Shape::Cylinder {
            center2,
            radii2,
            hollow,
            ..
        } => {
            if center2.iter().any(|c| !(-65536..=65536).contains(c))
                || radii2.iter().any(|r| !(1..=512).contains(r))
            {
                return Err(Error::Invalid("shape center/radii2".into()));
            }
            let mut min = std::array::from_fn(|a| (center2[a] - radii2[a]).div_euclid(2));
            let mut max = std::array::from_fn(|a| (center2[a] + radii2[a] + 1).div_euclid(2));
            if let Shape::Cylinder { axis, range, .. } = shape {
                min[axis.index()] = range[0];
                max[axis.index()] = range[1];
            }
            // Candidate volume may be wider than a document; output is never clipped.
            if (0..3).any(|a| max[a] <= min[a] || i64::from(max[a]) - i64::from(min[a]) > 513) {
                return Err(Error::Invalid("shape extent".into()));
            }
            (Region { min, max }, *hollow)
        }
        Shape::Line { .. } => unreachable!(),
    };
    let volume = (0..3)
        .map(|a| (region.max[a] - region.min[a]) as usize)
        .product();
    work.charge(volume)?;
    let inside = |p: Pos| -> bool {
        if !region.contains(p) {
            return false;
        }
        match shape {
            Shape::Box { .. } => true,
            Shape::Ellipsoid {
                center2, radii2, ..
            }
            | Shape::Cylinder {
                center2, radii2, ..
            } => {
                let axes: Vec<_> = (0..3)
                    .filter(|a| !matches!(shape,Shape::Cylinder { axis,.. } if *a==axis.index()))
                    .collect();
                let denom: i128 = axes.iter().map(|a| i128::from(radii2[*a]).pow(2)).product();
                axes.iter()
                    .map(|a| {
                        i128::from(2 * p[*a] + 1 - center2[*a]).pow(2)
                            * (denom / i128::from(radii2[*a]).pow(2))
                    })
                    .sum::<i128>()
                    <= denom
            }
            _ => false,
        }
    };
    Ok(region
        .cells()
        .filter(|p| {
            inside(*p)
                && (!hollow
                    || NEIGHBORS
                        .iter()
                        .any(|d| !inside(std::array::from_fn(|a| p[a] + d[a]))))
        })
        .collect())
}
// Exact rational boundary times: numerator odd, denominator twice |delta|.
fn line_cells(from: Pos, to: Pos, work: &mut Work) -> Result<Vec<Pos>> {
    for p in [from, to] {
        if p.iter().any(|v| !(-32768..32768).contains(v)) {
            return Err(Error::OutOfBounds(p));
        }
    }
    let delta: Pos = std::array::from_fn(|a| to[a] - from[a]);
    let mut p = from;
    let mut next = [1i64; 3];
    let mut points = BTreeSet::from([p]);
    work.charge(1)?;
    while p != to {
        let axis = (0..3)
            .filter(|a| delta[*a] != 0 && p[*a] != to[*a])
            .min_by(|a, b| {
                (next[*a] * i64::from(delta[*b].abs()))
                    .cmp(&(next[*b] * i64::from(delta[*a].abs())))
            })
            .expect("nonterminal line");
        let tied: Vec<_> = (0..3)
            .filter(|a| {
                delta[*a] != 0
                    && p[*a] != to[*a]
                    && next[*a] * i64::from(delta[axis].abs())
                        == next[axis] * i64::from(delta[*a].abs())
            })
            .collect();
        for mask in 1..(1 << tied.len()) {
            let mut q = p;
            for (i, a) in tied.iter().enumerate() {
                if mask & (1 << i) != 0 {
                    q[*a] += delta[*a].signum();
                }
            }
            work.charge(1)?;
            points.insert(q);
        }
        for a in tied {
            p[a] += delta[a].signum();
            next[a] += 2;
        }
    }
    Ok(points.into_iter().collect())
}

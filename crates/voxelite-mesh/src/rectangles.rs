use crate::{Mesh, MeshError, MeshResult};

impl Mesh {
    /// Consume a row-major face mask in row/column order, growing equal faces
    /// across columns first, then rows. `empty` marks absent or consumed faces.
    ///
    /// `mergeable` may retain a face as a unit rectangle (for example, at an AO
    /// discontinuity). Otherwise only faces equal to that entire value merge.
    /// `emit` receives the rectangle's column/row origin, width/height and face.
    /// No allocation or renderer-specific data is required by the merger.
    ///
    /// Returns an invalid-settings error, without consuming the mask, if its
    /// length differs from the product of `size`. Empty masks are valid. Emission
    /// errors stop immediately; the current and earlier rectangles have already
    /// been cleared, and any effects of earlier emissions belong to the caller.
    pub fn greedy_rectangles<T: Copy + Eq>(
        mask: &mut [T],
        size: [u16; 2],
        empty: T,
        mergeable: impl Fn(T) -> bool,
        mut emit: impl FnMut([u16; 2], [u16; 2], T) -> MeshResult<()>,
    ) -> MeshResult<()> {
        let [width, height] = size.map(usize::from);
        if mask.len() != width * height {
            return Err(MeshError::Invalid(
                "face mask dimensions do not match".into(),
            ));
        }
        for y in 0..height {
            for x in 0..width {
                let face = mask[y * width + x];
                if face == empty {
                    continue;
                }
                let mut w = 1;
                let mut h = 1;
                if mergeable(face) {
                    while x + w < width && mask[y * width + x + w] == face {
                        w += 1;
                    }
                    while y + h < height && (0..w).all(|dx| mask[(y + h) * width + x + dx] == face)
                    {
                        h += 1;
                    }
                }
                for dy in 0..h {
                    mask[(y + dy) * width + x..(y + dy) * width + x + w].fill(empty);
                }
                emit([x as u16, y as u16], [w as u16, h as u16], face)?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests;

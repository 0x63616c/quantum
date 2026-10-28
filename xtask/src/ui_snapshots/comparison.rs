//! Exact geometry, <=5/255 per-channel fuzz, and <=1/2000 changed pixels.
use super::Result;
use std::{
    fs::File,
    io::{BufReader, BufWriter},
    path::Path,
};

pub(super) struct Image {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
}

impl Image {
    pub fn read(path: &Path) -> Result<Self> {
        let mut decoder = png::Decoder::new(BufReader::new(File::open(path)?));
        decoder.set_transformations(png::Transformations::EXPAND | png::Transformations::STRIP_16);
        let mut reader = decoder.read_info()?;
        let info = reader.info();
        if info.width == 0 || info.height == 0 || info.width > 4096 || info.height > 4096 {
            return Err(format!("invalid snapshot dimensions: {}", path.display()).into());
        }
        let mut bytes = vec![
            0;
            reader
                .output_buffer_size()
                .ok_or("PNG exceeds size limit")?
        ];
        let output = reader.next_frame(&mut bytes)?;
        bytes.truncate(output.buffer_size());
        let rgba = match output.color_type {
            png::ColorType::Rgba => bytes,
            png::ColorType::Rgb => bytes
                .as_chunks::<3>()
                .0
                .iter()
                .flat_map(|p| [p[0], p[1], p[2], 255])
                .collect(),
            _ => return Err("snapshot must be RGB or RGBA PNG".into()),
        };
        Ok(Self {
            width: output.width,
            height: output.height,
            rgba,
        })
    }

    pub fn write(&self, path: &Path) -> Result<()> {
        let mut encoder =
            png::Encoder::new(BufWriter::new(File::create(path)?), self.width, self.height);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        encoder.write_header()?.write_image_data(&self.rgba)?;
        Ok(())
    }

    fn pixel(&self, x: u32, y: u32) -> Option<&[u8]> {
        if x >= self.width || y >= self.height {
            return None;
        }
        let offset = ((y * self.width + x) * 4) as usize;
        self.rgba.get(offset..offset + 4)
    }
}

pub(super) struct Comparison {
    pub passed: bool,
    pub changed: u64,
    pub diff: Image,
}

pub(super) fn compare(actual: &Image, expected: &Image) -> Comparison {
    let same_size = (actual.width, actual.height) == (expected.width, expected.height);
    let width = actual.width.max(expected.width);
    let height = actual.height.max(expected.height);
    let mut diff = Image {
        width,
        height,
        rgba: Vec::with_capacity((width * height * 4) as usize),
    };
    let mut changed = 0;
    for y in 0..height {
        for x in 0..width {
            let different = match (actual.pixel(x, y), expected.pixel(x, y)) {
                (Some(a), Some(b)) => a.iter().zip(b).any(|(&a, &b)| a.abs_diff(b) > 5),
                _ => true,
            };
            if different {
                changed += 1;
            }
            diff.rgba.extend(if different {
                [255, 0, 128, 255]
            } else {
                [0, 0, 0, 255]
            });
        }
    }
    Comparison {
        passed: same_size && changed * 2000 <= u64::from(width) * u64::from(height),
        changed,
        diff,
    }
}

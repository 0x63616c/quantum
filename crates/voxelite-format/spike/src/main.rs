//! Repeatable, isolated load/save experiment. See ../README.md for methodology.
mod codec;
mod scenes;

use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    fs::{self, File},
    hint::black_box,
    path::{Path, PathBuf},
    time::Instant,
};
use voxelite_core::Limits;
use voxelite_format::{Project, session};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(Clone, Copy, Debug)]
enum Codec {
    Reader,
    Slice,
    Archive,
    Packed,
}
impl Codec {
    const ALL: [Self; 4] = [Self::Reader, Self::Slice, Self::Archive, Self::Packed];
    fn name(self) -> &'static str {
        match self {
            Self::Reader => "json-reader",
            Self::Slice => "json-slice",
            Self::Archive => "rkyv",
            Self::Packed => "packed-control",
        }
    }
    fn named(name: &str) -> Result<Self> {
        Self::ALL
            .into_iter()
            .find(|codec| codec.name() == name)
            .ok_or_else(|| "unknown codec".into())
    }
    fn decode(self, bytes: &[u8]) -> Result<Project> {
        match self {
            Self::Reader => Ok(voxelite_format::read(bytes)?),
            Self::Slice => Ok(voxelite_format::from_bytes(bytes)?),
            Self::Archive => codec::decode(bytes),
            Self::Packed => codec::decode_packed(bytes),
        }
    }
    fn encode(self, project: &Project) -> Result<Vec<u8>> {
        match self {
            Self::Reader | Self::Slice => Ok(project.to_bytes()?),
            Self::Archive => codec::encode(project),
            Self::Packed => codec::encode_packed(project),
        }
    }
}

#[derive(Default, Serialize)]
struct Samples {
    ms: Vec<f64>,
    cpu_ms: Vec<f64>,
    // Linux /proc/self/io counts reads serviced below the guest page cache.
    read_bytes: Vec<u64>,
}
impl Samples {
    fn record<T>(&mut self, action: impl FnOnce() -> Result<T>) -> Result<()> {
        let before = physical_reads();
        let cpu_start = process_cpu_ms();
        let start = Instant::now();
        let result = action()?;
        self.ms.push(start.elapsed().as_secs_f64() * 1000.0);
        self.cpu_ms.push(process_cpu_ms() - cpu_start);
        self.read_bytes
            .push(physical_reads().saturating_sub(before));
        // Destruction is deliberately outside the timed interval.
        black_box(result);
        Ok(())
    }
    fn report(&self) -> serde_json::Value {
        let mut sorted = self.ms.clone();
        sorted.sort_by(f64::total_cmp);
        let mut cpu = self.cpu_ms.clone();
        cpu.sort_by(f64::total_cmp);
        serde_json::json!({
            "median_ms": sorted[sorted.len() / 2],
            "median_cpu_ms": cpu.get(cpu.len() / 2),
            "min_ms": sorted[0],
            "max_ms": sorted[sorted.len() - 1],
            "samples": self,
        })
    }
}

fn process_cpu_ms() -> f64 {
    let time = rustix::time::clock_gettime(rustix::time::ClockId::ProcessCPUTime);
    time.tv_sec as f64 * 1000.0 + time.tv_nsec as f64 / 1_000_000.0
}

fn physical_reads() -> u64 {
    fs::read_to_string("/proc/self/io")
        .ok()
        .and_then(|s| {
            s.lines()
                .find_map(|l| l.strip_prefix("read_bytes: ")?.parse().ok())
        })
        .unwrap_or(0)
}

fn evict(path: &Path) -> Result<()> {
    let file = File::open(path)?;
    file.sync_all()?;
    // Per-file guest cache eviction only; no global drop_caches or unsafe code.
    rustix::fs::fadvise(&file, 0, None, rustix::fs::Advice::DontNeed)?;
    Ok(())
}

fn write_fixture(path: &Path, bytes: &[u8]) -> Result<()> {
    fs::write(path, bytes)?;
    File::open(path)?.sync_all()?;
    Ok(())
}

fn open(path: &Path, codec: Codec) -> Result<session::DocumentSession> {
    // Mirrors DocumentSession::open, including bounded IO, engine validation and
    // the exact-byte fingerprint. Only the codec is replaced in this experiment.
    let path = path.canonicalize()?;
    let bytes = session::read(&path)?;
    let project = codec.decode(&bytes)?;
    let session = session::DocumentSession::new(project, None, Limits::default())?;
    black_box(session::fingerprint(&bytes));
    Ok(session)
}

fn durable_save(path: &Path, project: &Project, codec: Codec) -> Result<usize> {
    let bytes = codec.encode(project)?;
    session::write_new(path, &bytes)?;
    black_box(session::fingerprint(&bytes));
    Ok(bytes.len())
}

fn measure(
    rows: &mut BTreeMap<String, Samples>,
    name: &str,
    runs: usize,
    mut action: impl FnMut() -> Result<()>,
) -> Result<()> {
    action()?;
    for _ in 0..runs {
        rows.entry(name.into()).or_default().record(&mut action)?;
    }
    Ok(())
}

// Diagnostic parser-only control: consume exactly the same JSON voxel arrays,
// retaining counts instead of chunks. Does NOT constitute a validated load.
#[derive(Deserialize)]
struct Numbers {
    layers: Vec<NumberLayer>,
}
#[derive(Deserialize)]
struct NumberLayer {
    voxels: Count,
}
struct Count(usize);
impl<'de> Deserialize<'de> for Count {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> std::result::Result<Self, D::Error> {
        struct Counter;
        impl<'de> serde::de::Visitor<'de> for Counter {
            type Value = Count;
            fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str("voxel tuples")
            }
            fn visit_seq<A: serde::de::SeqAccess<'de>>(
                self,
                mut seq: A,
            ) -> std::result::Result<Count, A::Error> {
                let mut count = 0;
                while let Some(cell) = seq.next_element::<[i32; 4]>()? {
                    black_box(cell);
                    count += 1;
                }
                Ok(Count(count))
            }
        }
        d.deserialize_seq(Counter)
    }
}

fn profile(
    project: &Project,
    bytes: &[u8],
    runs: usize,
    rows: &mut BTreeMap<String, Samples>,
) -> Result<()> {
    measure(rows, "phase/document-validate", runs, || {
        Ok(black_box(project.document.validate())?)
    })?;
    measure(rows, "phase/open-engine", runs, || {
        black_box(project.open_engine(Limits::default())?);
        Ok(())
    })?;
    measure(rows, "phase/sha256", runs, || {
        black_box(session::fingerprint(bytes));
        Ok(())
    })?;
    let tuples: Vec<_> = project.document.content.layers[0]
        .voxels
        .sorted()
        .into_iter()
        .map(|([x, y, z], s)| [x, y, z, i32::from(s)])
        .collect();
    measure(
        rows,
        "phase/chunks-from-native-tuples-first-layer",
        runs,
        || {
            black_box(codec::volume(tuples.iter().copied())?);
            Ok(())
        },
    )?;
    for (label, reader) in [("reader", true), ("slice", false)] {
        measure(
            rows,
            &format!("phase/json-numbers-only-{label}"),
            runs,
            || {
                let parsed: Numbers = if reader {
                    serde_json::from_reader(std::io::BufReader::new(bytes))?
                } else {
                    serde_json::from_slice(bytes)?
                };
                black_box(parsed.layers.iter().map(|l| l.voxels.0).sum::<usize>());
                Ok(())
            },
        )?;
    }
    Ok(())
}

fn bench_case(name: &str, project: Project, dir: &Path, runs: usize) -> Result<serde_json::Value> {
    eprintln!("Measuring {name} ({runs} samples per metric)");
    let mut rows: BTreeMap<String, Samples> = BTreeMap::new();
    let expected = project.to_bytes()?;
    let mut inputs = Vec::new();
    for codec in Codec::ALL {
        let bytes = codec.encode(&project)?;
        let restored = codec.decode(&bytes)?;
        if restored.to_bytes()? != expected {
            return Err(format!("{name}: {} canonical mismatch", codec.name()).into());
        }
        let path = dir.join(format!("{name}-{}.bin", codec.name()));
        write_fixture(&path, &bytes)?;
        inputs.push((codec, bytes, path));
    }
    // Warm-up once, then rotate codec order each repetition to reduce drift bias.
    for (codec, bytes, _) in &inputs {
        black_box(codec.decode(bytes)?);
        black_box(codec.encode(&project)?);
    }
    for run in 0..runs {
        for index in 0..inputs.len() {
            let (codec, bytes, path) = &inputs[(index + run) % inputs.len()];
            let label = codec.name();
            rows.entry(format!("{label}/decode-memory"))
                .or_default()
                .record(|| codec.decode(bytes))?;
            rows.entry(format!("{label}/encode-memory"))
                .or_default()
                .record(|| codec.encode(&project))?;
            // Read outside the timer to establish a warm file cache.
            black_box(session::read(path)?);
            rows.entry(format!("{label}/open-warm"))
                .or_default()
                .record(|| open(path, *codec))?;
            evict(path)?;
            rows.entry(format!("{label}/open-cold-requested"))
                .or_default()
                .record(|| open(path, *codec))?;
            // Every durable save creates a fresh file, just like Save As to a
            // new destination; destination caching cannot avoid the syncs.
            let output = dir.join(format!("{name}-{label}-save-{run}.bin"));
            if output.exists() {
                fs::remove_file(&output)?;
            }
            rows.entry(format!("{label}/save-durable-new"))
                .or_default()
                .record(|| durable_save(&output, &project, *codec))?;
            fs::remove_file(output)?;
            // First serializer/save call in a fresh child. Launch and loading
            // the source model are excluded, matching an editor's Save action.
            // "Cold" does not mean cold disk writes: all outputs are new files.
            let input = &inputs[0].2;
            evict(input)?;
            let output = dir.join(format!("{name}-{label}-first-save-{run}.bin"));
            if output.exists() {
                fs::remove_file(&output)?;
            }
            let child = std::process::Command::new(std::env::current_exe()?)
                .arg("--first-save")
                .arg(label)
                .arg(input)
                .arg(&output)
                .output()?;
            if !child.status.success() {
                return Err(String::from_utf8_lossy(&child.stderr).into_owned().into());
            }
            let times: [f64; 2] = serde_json::from_slice(&child.stdout)?;
            for (phase, ms) in ["encode-first-process", "save-durable-first-process"]
                .into_iter()
                .zip(times)
            {
                rows.entry(format!("{label}/{phase}"))
                    .or_default()
                    .ms
                    .push(ms);
            }
            fs::remove_file(output)?;
        }
    }
    profile(&project, &expected, runs, &mut rows)?;
    let archived = &inputs[2].1;
    let staged = codec::stage(&project)?;
    measure(&mut rows, "phase/rkyv-bytecheck-access", runs, || {
        black_box(codec::access(archived)?);
        Ok(())
    })?;
    measure(&mut rows, "phase/rkyv-serialize-prestaged", runs, || {
        black_box(codec::archive(&staged)?);
        Ok(())
    })?;
    measure(&mut rows, "phase/rkyv-stage-sort", runs, || {
        black_box(codec::stage(&project)?);
        Ok(())
    })?;
    let sizes: BTreeMap<_, _> = inputs.iter().map(|(c, b, _)| (c.name(), b.len())).collect();
    Ok(serde_json::json!({
        "voxels": project.document.content.layers.iter().map(|l| l.voxels.len()).sum::<usize>(),
        "chunks": project.document.content.layers.iter().map(|l| l.voxels.chunk_count()).sum::<usize>(),
        "canonical_sha256": session::fingerprint(&expected),
        "bytes": sizes,
        "metrics": rows.iter().map(|(k, v)| (k.clone(), v.report())).collect::<BTreeMap<_, _>>()
    }))
}

fn autumn(dir: &Path, runs: usize) -> Result<serde_json::Value> {
    let input =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../examples/autumn-workshop.voxl");
    let bytes = session::read(&input)?;
    let project = voxelite_format::read(bytes.as_slice())?;
    let mut rows: BTreeMap<String, Samples> = BTreeMap::new();
    for (label, codec) in [("json-reader", Codec::Reader), ("json-slice", Codec::Slice)] {
        measure(&mut rows, &format!("{label}/decode-memory"), runs, || {
            black_box(codec.decode(&bytes)?);
            Ok(())
        })?;
    }
    measure(&mut rows, "session-open", runs, || {
        black_box(session::DocumentSession::open(&input, Limits::default())?);
        Ok(())
    })?;
    profile(&project, &bytes, runs, &mut rows)?;
    fs::write(
        dir.join("autumn-profile.json"),
        serde_json::to_vec_pretty(
            &rows
                .iter()
                .map(|(k, v)| (k.clone(), v.report()))
                .collect::<BTreeMap<_, _>>(),
        )?,
    )?;
    // Separate content-only case, explicitly omitting recipe and named version.
    // The intact original is measured above; the prototype rejects it.
    let mut document = project.document.clone();
    document.versions.clear();
    let content = Project::new(document)?;
    bench_case("autumn-content", content, dir, runs)
}

fn main() -> Result<()> {
    let mut args = std::env::args().skip(1);
    let first = args.next().unwrap_or_else(|| "/tmp/payload-spike".into());
    if first == "--first-save" {
        let codec = Codec::named(&args.next().ok_or("missing codec")?)?;
        let input = args.next().ok_or("missing input")?;
        let output = args.next().ok_or("missing output")?;
        let project = voxelite_format::from_bytes(&session::read(Path::new(&input))?)?;
        let start = Instant::now();
        let bytes = codec.encode(&project)?;
        let encode_ms = start.elapsed().as_secs_f64() * 1000.0;
        session::write_new(Path::new(&output), &bytes)?;
        black_box(session::fingerprint(&bytes));
        let durable_ms = start.elapsed().as_secs_f64() * 1000.0;
        println!("{}", serde_json::to_string(&[encode_ms, durable_ms])?);
        return Ok(());
    }
    let dir = PathBuf::from(first);
    let runs: usize = args.next().unwrap_or_else(|| "7".into()).parse()?;
    if runs < 3 || runs.is_multiple_of(2) || args.next().is_some() {
        return Err("usage: payload-spike [output-directory] [odd runs >= 3]".into());
    }
    fs::create_dir_all(&dir)?;
    let mut result = serde_json::Map::new();
    result.insert(
        "small".into(),
        bench_case("small", scenes::terrain(16, 16, 2048, "small")?, &dir, runs)?,
    );
    result.insert(
        "terrain1m".into(),
        bench_case(
            "terrain1m",
            scenes::terrain(128, 128, 1_040_000, "terrain1m")?,
            &dir,
            runs,
        )?,
    );
    result.insert("autumn-content".into(), autumn(&dir, runs)?);
    fs::write(
        dir.join("results.json"),
        serde_json::to_vec_pretty(&result)?,
    )?;
    eprintln!("Results: {}", dir.join("results.json").display());
    Ok(())
}

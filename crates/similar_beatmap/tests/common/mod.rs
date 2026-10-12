//! 集成测试用的合成数据集构造器。
//!
//! 文件格式与 `osu-difficulty-runtime` 完全一致（bincode 布局 + 校验和 sidecar），
//! 参考该 crate 自带测试的 fixture，但只构造查询路径需要的文件：
//!
//! - standard：特征文件 + 归一化器 + SQLite 元数据 + HNSW 索引（含 `.sha256`）
//! - mania：特征文件 + Mod 特征 + 归一化器 + bucket 索引（含 `.sha256`）+ SQLite
//!
//! 有了它，`/similar` 的解析、查询、渲染可以在无网络、无真实数据集的情况下端到端跑通。

#![allow(dead_code)]

use std::fs::{self, File};
use std::io::{Seek, Write};
use std::path::{Path, PathBuf};

use hnsw::{Hnsw, Params, Searcher};
use osu_difficulty_runtime::{
    ANALYZER_ALGORITHM_ID, ANALYZER_VERSION, BaseFeatures, BeatmapFeatureRecord, DifficultyVector,
    MANIA_ANALYZER_SNAPSHOT, MANIA_ANALYZER_VERSION, MANIA_NORMALIZATION_VERSION,
    ManiaBaseFeatures, ManiaDifficultyVector, ManiaFeatureRecord, ManiaGameMod,
    ManiaModFeatureRecord, ManiaModeFamily, ManiaPattern, ManiaStyleVector,
    OVERLAP_ALGORITHM_VERSION, READING_ALGORITHM_VERSION, ROSU_PP_VERSION,
};
use rand_pcg::Pcg64;
use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use space::Metric;

/// standard 特征文件头。
const STANDARD_HEADER: &[u8; 7] = b"ODLNORM";
/// standard 特征文件头长度。
const STANDARD_HEADER_LEN: usize = 32;
/// standard 特征文件格式版本。
const FEATURE_FORMAT_VERSION: u32 = 1;
/// mania 特征文件头。
const MANIA_HEADER: &[u8; 8] = b"ODLMAN1\0";
/// mania Mod 特征文件头。
const MANIA_MOD_HEADER: &[u8; 8] = b"ODLMMV1\0";
/// 归一化 / 分析版本：fixture 统一用 1。
const VERSION: u32 = 1;

/// 在 `root/standard` 下构造 standard 数据集，返回目录。
///
/// 4 张谱面：`10`(set 1) 与 `20`/`21`(set 2) 难度接近，`30`(set 3) 明显更难。
/// 查询 `10` 时 `21` 会因同谱面集去重被 `20` 挤掉，`30` 作为次优候选出现。
pub fn standard_dataset(root: &Path) -> PathBuf {
    let dir = root.join("standard");
    fs::create_dir_all(dir.join("indexes")).expect("index dir");
    fs::create_dir_all(dir.join("normalizers")).expect("normalizer dir");

    let records = [
        standard_record(10, 1, [0.10; 5], 180.0),
        standard_record(20, 2, [0.11; 5], 181.0),
        standard_record(21, 2, [0.12; 5], 182.0),
        standard_record(30, 3, [0.80; 5], 240.0),
    ];
    let record_size =
        bincode::serialized_size(&BeatmapFeatureRecord::default()).expect("record size") as usize;

    // 特征文件：32 字节头 + 定长记录
    let mut features = File::create(dir.join("features-v1.bin")).expect("feature file");
    let mut header = [0_u8; STANDARD_HEADER_LEN];
    header[..STANDARD_HEADER.len()].copy_from_slice(STANDARD_HEADER);
    header[8..12].copy_from_slice(&FEATURE_FORMAT_VERSION.to_le_bytes());
    features.write_all(&header).expect("feature header");
    for record in &records {
        bincode::serialize_into(&mut features, record).expect("feature record");
    }
    features.flush().expect("flush features");

    // 归一化器：两段式分位表（0.0 / 1.0）
    write_bincode(
        &dir.join("normalizers/v1.bin"),
        &StandardNormalizer {
            version: VERSION,
            quantiles: std::array::from_fn(|_| vec![0.0, 1.0]),
        },
    );

    // SQLite 元数据：beatmaps（带星数）+ analyses + analysis_versions
    let connection = Connection::open(dir.join("metadata.sqlite")).expect("metadata");
    connection
        .execute_batch(
            "CREATE TABLE beatmaps (
                beatmap_id INTEGER PRIMARY KEY, beatmapset_id INTEGER NOT NULL,
                checksum TEXT NOT NULL, artist TEXT NOT NULL, title TEXT NOT NULL,
                version TEXT NOT NULL, creator TEXT NOT NULL, online_url TEXT NOT NULL,
                star_rating REAL, star_section INTEGER, updated_at INTEGER NOT NULL
             );
             CREATE TABLE analyses (
                beatmap_id INTEGER NOT NULL, analyzer_version INTEGER NOT NULL,
                normalization_version INTEGER NOT NULL, normalized_offset INTEGER,
                status INTEGER NOT NULL
             );
             CREATE TABLE analysis_versions (
                analyzer_version INTEGER PRIMARY KEY, algorithm_id TEXT NOT NULL,
                rosu_pp_version TEXT NOT NULL, reading_version TEXT NOT NULL,
                overlap_version TEXT NOT NULL
             );",
        )
        .expect("schema");
    connection
        .execute(
            "INSERT INTO analysis_versions VALUES (?1,?2,?3,?4,?5)",
            params![
                ANALYZER_VERSION,
                ANALYZER_ALGORITHM_ID,
                ROSU_PP_VERSION,
                READING_ALGORITHM_VERSION,
                OVERLAP_ALGORITHM_VERSION
            ],
        )
        .expect("algorithm row");
    for (index, record) in records.iter().enumerate() {
        let stars = standard_stars(record.beatmap_id);
        connection
            .execute(
                "INSERT INTO beatmaps VALUES (?1,?2,?3,'Fixture Artist',?4,?5,'Mapper',?6,?7,?8,?9)",
                params![
                    record.beatmap_id as i64,
                    record.beatmapset_id as i64,
                    format!("checksum-{}", record.beatmap_id),
                    format!("Title {}", record.beatmap_id),
                    format!("Diff {}", record.beatmap_id),
                    format!("https://osu.ppy.sh/b/{}", record.beatmap_id),
                    stars,
                    (stars * 10.0).floor() as i64,
                    1_700_000_000_i64 + index as i64,
                ],
            )
            .expect("beatmap row");
        connection
            .execute(
                "INSERT INTO analyses VALUES (?1,?2,1,?3,2)",
                params![
                    record.beatmap_id as i64,
                    ANALYZER_VERSION,
                    STANDARD_HEADER_LEN as i64 + index as i64 * record_size as i64
                ],
            )
            .expect("analysis row");
    }
    drop(connection);

    // HNSW 主索引（含 .sha256 校验文件）
    let mut graph = Graph::new_params(FixtureMetric, Params::new().ef_construction(200));
    let mut searcher = Searcher::new();
    let mut labels = Vec::new();
    for record in &records {
        graph.insert(record.difficulty.as_array(), &mut searcher);
        labels.push(record.beatmap_id);
    }
    let bytes = bincode::serialize(&StandardIndex {
        labels,
        graph,
        normalization_version: VERSION,
    })
    .expect("serialize index");
    let index_path = dir.join("indexes/difficulty-main.hnsw");
    fs::write(&index_path, &bytes).expect("write index");
    fs::write(
        format!("{}.sha256", index_path.to_string_lossy()),
        hex::encode(Sha256::digest(&bytes)),
    )
    .expect("write index checksum");

    dir
}

/// 在 `root/mania` 下构造 mania 数据集，返回目录。
///
/// 5 张谱面：4K 三张（`10`/`11` 同集、`20` 另一集）、6K 一张（`60`）、7K 一张（`70`），
/// 用来验证键数隔离与 Mod 池（DT/HT 记录由 Mod 特征文件提供）。
pub fn mania_dataset(root: &Path) -> PathBuf {
    let dir = root.join("mania");
    fs::create_dir_all(dir.join("indexes")).expect("index dir");
    fs::create_dir_all(dir.join("normalizers")).expect("normalizer dir");

    let records = mania_records();

    // 特征文件 + 各记录的归一化偏移
    let mut offsets = Vec::new();
    let mut features = File::create(dir.join("mania-features-v1.bin")).expect("feature file");
    features.write_all(MANIA_HEADER).expect("feature header");
    for record in &records {
        offsets.push(features.stream_position().expect("feature offset"));
        bincode::serialize_into(&mut features, record).expect("feature record");
    }
    features.flush().expect("flush features");

    // Mod 特征：DT / HT 变体（改 BPM 与 NPS，其余沿用 NoMod）
    let mut mods = File::create(dir.join("mania-mod-features-v1.bin")).expect("mod file");
    mods.write_all(MANIA_MOD_HEADER).expect("mod header");
    for record in &records {
        for (game_mod, scale) in [(ManiaGameMod::Dt, 1.5_f32), (ManiaGameMod::Ht, 0.75_f32)] {
            let mut variant = *record;
            variant.base.bpm *= scale;
            variant.base.avg_nps *= scale;
            bincode::serialize_into(
                &mut mods,
                &ManiaModFeatureRecord {
                    beatmap_id: variant.beatmap_id,
                    game_mod,
                    record: variant,
                },
            )
            .expect("mod record");
        }
    }
    mods.flush().expect("flush mods");

    // 归一化器：每个键数一套分位表（cohort 规模必须与记录数一致）
    let key = |key_count: u8, count: usize| ManiaKeyNormalizer {
        key_count,
        axes: std::array::from_fn(|_| (0..count).map(|value| value as f32).collect()),
        overall: if count == 1 {
            vec![0.5]
        } else {
            (0..count)
                .map(|value| value as f32 / (count - 1) as f32)
                .collect()
        },
    };
    write_bincode(
        &dir.join("normalizers/mania-v1.bin"),
        &ManiaNormalizerFixture {
            version: VERSION,
            analyzer_version: MANIA_ANALYZER_VERSION,
            keys: vec![key(4, 3), key(6, 1), key(7, 1)],
        },
    );

    // bucket 索引：按 (键数, 难度档) 分桶，必须恰好覆盖全部记录
    let mut buckets: Vec<ManiaBucketFixture> = Vec::new();
    for (index, record) in records.iter().enumerate() {
        match buckets.iter_mut().find(|bucket| {
            bucket.key_count == record.key_count && bucket.difficulty_band == record.difficulty_band
        }) {
            Some(bucket) => bucket.entries.push(entry(record, offsets[index])),
            None => buckets.push(ManiaBucketFixture {
                key_count: record.key_count,
                difficulty_band: record.difficulty_band,
                entries: vec![entry(record, offsets[index])],
            }),
        }
    }
    let index_bytes = bincode::serialize(&ManiaIndexFixture {
        normalization_version: VERSION,
        analyzer_version: MANIA_ANALYZER_VERSION,
        buckets,
    })
    .expect("serialize mania index");
    fs::write(dir.join("indexes/mania-v1.buckets"), &index_bytes).expect("write mania index");
    fs::write(
        dir.join("indexes/mania-v1.buckets.sha256"),
        hex::encode(Sha256::digest(&index_bytes)),
    )
    .expect("write mania index checksum");

    // SQLite 元数据 + 扫描状态
    let connection = Connection::open(dir.join("mania-metadata.sqlite")).expect("metadata");
    connection
        .execute_batch(
            "CREATE TABLE mania_beatmaps (
                beatmap_id INTEGER PRIMARY KEY,
                beatmapset_id INTEGER NOT NULL,
                checksum TEXT NOT NULL,
                artist TEXT NOT NULL,
                title TEXT NOT NULL,
                version TEXT NOT NULL,
                creator TEXT NOT NULL,
                online_url TEXT NOT NULL,
                key_count INTEGER NOT NULL,
                mode_family INTEGER NOT NULL,
                dominant_pattern INTEGER NOT NULL,
                updated_at INTEGER NOT NULL DEFAULT (unixepoch())
             );
             CREATE TABLE mania_analyses (
                beatmap_id INTEGER NOT NULL,
                analyzer_version INTEGER NOT NULL,
                normalization_version INTEGER NOT NULL DEFAULT 0,
                raw_offset INTEGER NOT NULL,
                normalized_offset INTEGER,
                status INTEGER NOT NULL,
                PRIMARY KEY (beatmap_id, analyzer_version)
             );
             CREATE TABLE mania_state (key TEXT PRIMARY KEY, value TEXT NOT NULL);",
        )
        .expect("schema");
    for (index, record) in records.iter().enumerate() {
        connection
            .execute(
                "INSERT INTO mania_beatmaps(
                    beatmap_id,beatmapset_id,checksum,artist,title,version,creator,online_url,
                    key_count,mode_family,dominant_pattern
                 ) VALUES(?1,?2,?3,'Fixture Artist',?4,?5,'Mapper',?6,?7,?8,?9)",
                params![
                    record.beatmap_id as i64,
                    record.beatmapset_id as i64,
                    format!("{:064x}", record.beatmap_id),
                    format!("Mania Title {}", record.beatmap_id),
                    format!("Mania Diff {}", record.beatmap_id),
                    format!("https://osu.ppy.sh/b/{}", record.beatmap_id),
                    record.key_count as i64,
                    record.mode_family as i64,
                    record.dominant_pattern as i64,
                ],
            )
            .expect("beatmap row");
        connection
            .execute(
                "INSERT INTO mania_analyses(
                    beatmap_id,analyzer_version,normalization_version,raw_offset,
                    normalized_offset,status
                 ) VALUES(?1,?2,1,8,?3,2)",
                params![
                    record.beatmap_id as i64,
                    MANIA_ANALYZER_VERSION,
                    offsets[index] as i64
                ],
            )
            .expect("analysis row");
    }
    for (key, value) in [
        ("analyzer_snapshot", MANIA_ANALYZER_SNAPSHOT.to_owned()),
        ("scan_eligible", records.len().to_string()),
        ("scan_unsupported", "0".to_owned()),
        ("scan_failed", "0".to_owned()),
    ] {
        connection
            .execute(
                "INSERT INTO mania_state(key,value) VALUES(?1,?2)",
                params![key, value],
            )
            .expect("state row");
    }
    drop(connection);

    dir
}

/// 把数据集放到 `<模块目录>/standard`（handler 走的默认路径）。
pub fn install_standard(module_dir: &Path) -> PathBuf {
    fs::create_dir_all(module_dir).expect("module dir");
    standard_dataset(module_dir)
}

/// 把数据集放到 `<模块目录>/mania`。
pub fn install_mania(module_dir: &Path) -> PathBuf {
    fs::create_dir_all(module_dir).expect("module dir");
    mania_dataset(module_dir)
}

/// standard fixture 里的星数（与 SQLite 里的 star_rating 保持一致）。
fn standard_stars(beatmap_id: u64) -> f64 {
    match beatmap_id {
        10 => 6.0,
        20 => 6.1,
        21 => 6.2,
        30 => 8.0,
        other => other as f64 / 10.0,
    }
}

/// 构造一条 standard 特征记录。
fn standard_record(
    beatmap_id: u64,
    beatmapset_id: u64,
    difficulty: [f32; 5],
    bpm: f32,
) -> BeatmapFeatureRecord {
    BeatmapFeatureRecord {
        beatmap_id,
        beatmapset_id,
        difficulty: DifficultyVector::from_array(difficulty),
        base: BaseFeatures {
            bpm,
            ar: 9.0,
            cs: 4.0,
            od: 8.0,
            length_seconds: 120.0,
            object_density: 4.0,
            circle_ratio: 0.6,
            slider_ratio: 0.4,
            ..BaseFeatures::default()
        },
        analyzer_version: ANALYZER_VERSION,
        normalization_version: VERSION,
        ..BeatmapFeatureRecord::default()
    }
}

/// mania fixture 的记录集合（4K 三张、6K 一张、7K 一张）。
fn mania_records() -> Vec<ManiaFeatureRecord> {
    [(10, 1, 4), (11, 1, 4), (20, 2, 4), (60, 6, 6), (70, 7, 7)]
        .into_iter()
        .enumerate()
        .map(
            |(index, (beatmap_id, beatmapset_id, key_count))| ManiaFeatureRecord {
                beatmap_id,
                beatmapset_id,
                difficulty: ManiaDifficultyVector::from_array([0.5; 8]),
                style: ManiaStyleVector {
                    stream: 1.0,
                    ..ManiaStyleVector::default()
                },
                base: ManiaBaseFeatures {
                    bpm: 150.0 + index as f32,
                    length_seconds: 90.0,
                    active_length_seconds: 85.0,
                    note_count: 900.0,
                    row_count: 800.0,
                    avg_nps: 6.0,
                    peak_nps: 9.0,
                    ..ManiaBaseFeatures::default()
                },
                difficulty_percentile: 0.5,
                difficulty_band: 5,
                key_count,
                mode_family: ManiaModeFamily::Rc,
                dominant_pattern: ManiaPattern::Stream,
                analyzer_version: MANIA_ANALYZER_VERSION,
                normalization_version: MANIA_NORMALIZATION_VERSION,
            },
        )
        .collect()
}

/// bucket 索引条目。
fn entry(record: &ManiaFeatureRecord, offset: u64) -> ManiaBucketEntryFixture {
    ManiaBucketEntryFixture {
        beatmap_id: record.beatmap_id,
        beatmapset_id: record.beatmapset_id,
        mode_family: record.mode_family,
        normalized_offset: offset,
    }
}

/// bincode 写入一个 fixture 结构。
fn write_bincode<T: Serialize>(path: &Path, value: &T) {
    let bytes = bincode::serialize(value).expect("serialize fixture");
    fs::write(path, bytes).expect("write fixture");
}

/// standard 的 HNSW 索引布局（与 runtime 私有的 `IndexFile` 一致）。
#[derive(Serialize, Deserialize)]
struct StandardIndex {
    labels: Vec<u64>,
    graph: Graph,
    normalization_version: u32,
}

/// HNSW 图类型：必须与 runtime 的 `Hnsw<WeightedL2, [f32; 5], Pcg64, 16, 32>` 一致。
type Graph = Hnsw<FixtureMetric, [f32; 5], Pcg64, 16, 32>;

/// 与 runtime 私有 `WeightedL2` 布局一致的度量（单位结构体 + 相同距离）。
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
struct FixtureMetric;

impl Metric<[f32; 5]> for FixtureMetric {
    type Unit = u64;

    fn distance(&self, left: &[f32; 5], right: &[f32; 5]) -> Self::Unit {
        left.iter()
            .zip(right)
            .map(|(left, right)| (*left as f64 - *right as f64).powi(2))
            .sum::<f64>()
            .to_bits()
    }
}

/// standard 归一化器布局。
#[derive(Serialize, Deserialize)]
struct StandardNormalizer {
    version: u32,
    quantiles: [Vec<f32>; 5],
}

/// mania 归一化器布局。
#[derive(Serialize, Deserialize)]
struct ManiaNormalizerFixture {
    version: u32,
    analyzer_version: u32,
    keys: Vec<ManiaKeyNormalizer>,
}

/// mania 单个键数的分位表布局。
#[derive(Serialize, Deserialize)]
struct ManiaKeyNormalizer {
    key_count: u8,
    axes: [Vec<f32>; 8],
    overall: Vec<f32>,
}

/// mania bucket 索引布局。
#[derive(Serialize, Deserialize)]
struct ManiaIndexFixture {
    normalization_version: u32,
    analyzer_version: u32,
    buckets: Vec<ManiaBucketFixture>,
}

/// 单个 (键数, 难度档) 桶。
#[derive(Serialize, Deserialize)]
struct ManiaBucketFixture {
    key_count: u8,
    difficulty_band: u8,
    entries: Vec<ManiaBucketEntryFixture>,
}

/// 桶内条目。
#[derive(Serialize, Deserialize)]
struct ManiaBucketEntryFixture {
    beatmap_id: u64,
    beatmapset_id: u64,
    mode_family: ManiaModeFamily,
    normalized_offset: u64,
}

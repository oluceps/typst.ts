use std::path::PathBuf;

pub fn corpus_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fuzzers/corpora")
}

pub fn artifact_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/typst-artifacts")
}

pub fn package_renderer_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../packages/renderer")
}

#[cfg(feature = "web_artifacts")]
#[cfg(target_arch = "wasm32")]
pub mod embedded_artifact {

    pub const MAIN_ARTIFACT_JSON: &[u8] =
        include_bytes!("../../../fuzzers/corpora/math/main.artifact.json");
    pub const MAIN_ARTIFACT_IR: &[u8] =
        include_bytes!("../../../fuzzers/corpora/math/main.artifact.tir.bin");
    pub const LINE_1_ARTIFACT_JSON: &[u8] =
        include_bytes!("../../../fuzzers/corpora/visualize/line_1.artifact.json");
    pub const LINE_1_ARTIFACT_IR: &[u8] =
        include_bytes!("../../../fuzzers/corpora/visualize/line_1.artifact.tir.bin");
    pub const LINE_2_ARTIFACT_JSON: &[u8] =
        include_bytes!("../../../fuzzers/corpora/visualize/line_2.artifact.json");
    pub const LINE_2_ARTIFACT_IR: &[u8] =
        include_bytes!("../../../fuzzers/corpora/visualize/line_2.artifact.tir.bin");
    pub const PATH_1_ARTIFACT_JSON: &[u8] =
        include_bytes!("../../../fuzzers/corpora/visualize/path_1.artifact.json");
    pub const PATH_1_ARTIFACT_IR: &[u8] =
        include_bytes!("../../../fuzzers/corpora/visualize/path_1.artifact.tir.bin");
    pub const POLYGON_1_ARTIFACT_JSON: &[u8] =
        include_bytes!("../../../fuzzers/corpora/visualize/polygon_1.artifact.json");
    pub const POLYGON_1_ARTIFACT_IR: &[u8] =
        include_bytes!("../../../fuzzers/corpora/visualize/polygon_1.artifact.tir.bin");
}

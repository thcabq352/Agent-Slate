//! Project JSON model — field names mirror `src/shared/types.ts` (camelCase).

use serde::{Deserialize, Serialize};

// ---- Shot building blocks ----

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShotSpec {
    pub duration_sec: Option<f64>,
    pub fps: Option<f64>,
    pub aspect_ratio: Option<String>,
    pub lens: Option<String>,
    pub movement: Option<String>,
    pub size: Option<String>,
    pub angle: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BeatDirection {
    pub from: f64,
    pub to: f64,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PromptVersion {
    pub id: String,
    pub saved_at: String,
    pub label: String,
    pub prompt: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum TakeRating {
    Circled,
    Good,
    NoGood,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Take {
    pub id: String,
    pub logged_at: String,
    pub model: String,
    pub prompt: String,
    pub rating: TakeRating,
    pub notes: String,
    /// Absolute path to the generated still/video. Old projects omit this.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub media_path: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Variant {
    pub id: String,
    pub label: String,
    pub prompt: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Shot {
    pub id: String,
    pub name: String,
    pub intent: String,
    pub spec: ShotSpec,
    pub prompt: String,
    pub locked_lines: Vec<u32>,
    pub muted_lines: Vec<u32>,
    pub beat_sheet: Option<Vec<BeatDirection>>,
    pub target_model: Option<String>,
    pub max_chars: Option<u32>,
    pub variants: Vec<Variant>,
    pub history: Vec<PromptVersion>,
    pub takes: Vec<Take>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Scene {
    pub id: String,
    pub name: String,
    pub synopsis: String,
    pub shots: Vec<Shot>,
}

// ---- Sheets ----

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ScenarioTab {
    Cinematic,
    Interview,
    Fashion,
    FilmScene,
    Portrait,
    Street,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CharacterSheet {
    pub id: String,
    pub name: String,
    pub age: String,
    pub gender: String,
    pub ethnicity: String,
    pub face_features: String,
    pub hair: String,
    pub clothing: String,
    pub expression: String,
    pub eye_direction: String,
    pub mood: String,
    pub environment: String,
    pub key_light_side: String,
    pub lighting_mood: String,
    pub scenario: ScenarioTab,
    pub notes: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub images: Option<Vec<String>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ArtDeptKind {
    Prop,
    Wardrobe,
    Vehicle,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArtDeptSheet {
    pub id: String,
    pub kind: ArtDeptKind,
    pub name: String,
    pub description: String,
    pub materials: String,
    pub condition: String,
    pub era: String,
    pub distinctive: String,
    pub notes: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum InteriorExterior {
    Interior,
    Exterior,
    Both,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LocationSheet {
    pub id: String,
    pub name: String,
    pub interior_exterior: InteriorExterior,
    pub description: String,
    pub time_of_day: String,
    pub weather: String,
    pub architecture: String,
    pub textures: String,
    pub practical_lights: String,
    pub notes: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub images: Option<Vec<String>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum StyleProfileKind {
    Cinematographer,
    Director,
    Film,
    Series,
    Custom,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StyleProfile {
    pub id: String,
    pub source: String,
    pub kind: StyleProfileKind,
    pub tone: String,
    pub palette: String,
    pub lighting: String,
    pub lens_language: String,
    pub movement: String,
    pub blocking: String,
    pub editorial: String,
    pub notes: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub images: Option<Vec<String>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ElementSheet {
    pub lensing: String,
    pub lighting: String,
    pub palette: String,
    pub composition: String,
    pub movement: String,
    pub texture: String,
    pub mood: String,
    pub notes: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ReferenceKind {
    Image,
    Video,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Reference {
    pub id: String,
    pub path: String,
    pub kind: ReferenceKind,
    pub label: String,
    pub frames: Vec<String>,
    pub elements: Option<ElementSheet>,
    pub added_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SectionId {
    Subject,
    Composition,
    Lighting,
    Camera,
    Style,
    Mood,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CustomSetup {
    pub id: String,
    pub label: String,
    pub snippet: String,
    pub section: SectionId,
    pub tags: Vec<String>,
    pub favorite: bool,
}

// ---- Sound / copilot (optional on Project) ----

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum VocalsPreference {
    Instrumental,
    Vocals,
    Either,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MusicCue {
    pub id: String,
    pub name: String,
    pub scene_ref: String,
    pub intent: String,
    pub genre: String,
    pub mood: String,
    pub tempo: String,
    pub instrumentation: String,
    pub era: String,
    pub structure: String,
    pub vocals: VocalsPreference,
    pub lyric_theme: String,
    pub lyrics: String,
    pub duration_sec: Option<f64>,
    pub notes: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VoiceSheet {
    pub id: String,
    pub name: String,
    pub character_id: Option<String>,
    pub age_gender: String,
    pub accent: String,
    pub timbre: String,
    pub pitch: String,
    pub pacing: String,
    pub energy: String,
    pub texture: String,
    pub emotional_range: String,
    pub sample_line: String,
    pub notes: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ChatRole {
    User,
    Assistant,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatMsg {
    pub role: ChatRole,
    pub text: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub receipts: Option<Vec<String>>,
}

// ---- Project defaults / brain ----

/// Which engine powers agent tasks. `local` talks to any OpenAI-compatible
/// localhost server (Ollama, LM Studio, vLLM, llama.cpp, KoboldCpp, Jan…).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BrainBackend {
    Claude,
    Codex,
    Local,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectDefaults {
    pub aspect_ratio: String,
    pub fps: f64,
    pub duration_sec: f64,
    pub target_model: String,
    pub brain: BrainBackend,
    /// Optional override for the local server base URL. Empty = auto-detect.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub local_endpoint: Option<String>,
    /// Model id to use on the local server. Empty = first available.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub local_model: Option<String>,
}

// ---- Project ----

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Project {
    pub id: String,
    pub name: String,
    pub logline: String,
    pub world: String,
    pub defaults: ProjectDefaults,
    pub scenes: Vec<Scene>,
    pub characters: Vec<CharacterSheet>,
    pub art_dept: Vec<ArtDeptSheet>,
    pub locations: Vec<LocationSheet>,
    pub lookbook: Vec<StyleProfile>,
    pub references: Vec<Reference>,
    pub my_setups: Vec<CustomSetup>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub music: Option<Vec<MusicCue>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub voices: Option<Vec<VoiceSheet>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub copilot: Option<Vec<ChatMsg>>,
    pub created_at: String,
    pub updated_at: String,
}

/// Create a new empty project. Mirrors `src/main/projects.ts` `newProject`.
pub fn new_project(name: &str) -> Project {
    let now = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true);
    Project {
        id: uuid::Uuid::new_v4().to_string(),
        name: name.to_string(),
        logline: String::new(),
        world: String::new(),
        defaults: ProjectDefaults {
            aspect_ratio: "16:9".to_string(),
            fps: 24.0,
            duration_sec: 8.0,
            target_model: "seedance-2".to_string(),
            brain: BrainBackend::Claude,
            local_endpoint: None,
            local_model: None,
        },
        scenes: Vec::new(),
        characters: Vec::new(),
        art_dept: Vec::new(),
        locations: Vec::new(),
        lookbook: Vec::new(),
        references: Vec::new(),
        my_setups: Vec::new(),
        music: Some(Vec::new()),
        voices: Some(Vec::new()),
        copilot: None,
        created_at: now.clone(),
        updated_at: now,
    }
}

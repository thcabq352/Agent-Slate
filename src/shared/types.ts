// Shared data model for Slate — used by main, preload, and renderer.

export type SectionId = 'subject' | 'composition' | 'lighting' | 'camera' | 'style' | 'mood'

export const SECTION_ORDER: SectionId[] = ['subject', 'composition', 'lighting', 'camera', 'style', 'mood']

export const SECTION_LABELS: Record<SectionId, string> = {
  subject: 'Subject',
  composition: 'Composition',
  lighting: 'Lighting',
  camera: 'Camera',
  style: 'Style',
  mood: 'Mood'
}

export interface ShotSpec {
  durationSec: number | null
  fps: number | null
  aspectRatio: string | null
  lens: string | null
  movement: string | null
  size: string | null
  angle: string | null
}

export interface BeatDirection {
  from: number
  to: number
  text: string
}

export interface PromptVersion {
  id: string
  savedAt: string
  label: string
  prompt: string
}

export interface Take {
  id: string
  loggedAt: string
  model: string
  prompt: string
  rating: 'circled' | 'good' | 'no-good'
  notes: string
  /** Absolute path to the generated still/video. Older projects may omit this. */
  mediaPath?: string | null
}

export interface Variant {
  id: string
  label: string
  prompt: string
}

export interface Shot {
  id: string
  name: string
  intent: string
  spec: ShotSpec
  prompt: string
  lockedLines: number[]
  mutedLines: number[]
  beatSheet: BeatDirection[] | null
  targetModel: string | null
  maxChars: number | null
  variants: Variant[]
  history: PromptVersion[]
  takes: Take[]
  createdAt: string
  updatedAt: string
}

export interface Scene {
  id: string
  name: string
  synopsis: string
  shots: Shot[]
}

export type ScenarioTab = 'cinematic' | 'interview' | 'fashion' | 'film-scene' | 'portrait' | 'street'

export interface CharacterSheet {
  id: string
  name: string
  age: string
  gender: string
  ethnicity: string
  faceFeatures: string
  hair: string
  clothing: string
  expression: string
  eyeDirection: string
  mood: string
  environment: string
  keyLightSide: string
  lightingMood: string
  scenario: ScenarioTab
  notes: string
  /** Continuity stills — image paths harvested from circled takes or clips. */
  images?: string[]
}

export type ArtDeptKind = 'prop' | 'wardrobe' | 'vehicle'

export interface ArtDeptSheet {
  id: string
  kind: ArtDeptKind
  name: string
  description: string
  materials: string
  condition: string
  era: string
  distinctive: string
  notes: string
}

export interface LocationSheet {
  id: string
  name: string
  interiorExterior: 'interior' | 'exterior' | 'both'
  description: string
  timeOfDay: string
  weather: string
  architecture: string
  textures: string
  practicalLights: string
  notes: string
  images?: string[]
}

export interface StyleProfile {
  id: string
  source: string
  kind: 'cinematographer' | 'director' | 'film' | 'series' | 'custom'
  tone: string
  palette: string
  lighting: string
  lensLanguage: string
  movement: string
  blocking: string
  editorial: string
  notes: string
  images?: string[]
}

export interface ElementSheet {
  lensing: string
  lighting: string
  palette: string
  composition: string
  movement: string
  texture: string
  mood: string
  notes: string
}

export interface CircledTake {
  project: string
  shot: string | null
  mediaPath: string
  fileName: string
  rating: number
  inSec: number | null
  outSec: number | null
}

export interface Reference {
  id: string
  path: string
  kind: 'image' | 'video'
  label: string
  frames: string[]
  elements: ElementSheet | null
  addedAt: string
}

export interface CustomSetup {
  id: string
  label: string
  snippet: string
  section: SectionId
  tags: string[]
  favorite: boolean
}

export interface ProjectDefaults {
  aspectRatio: string
  fps: number
  durationSec: number
  targetModel: string
  brain: BrainBackend
  /** Optional override for the local server base URL (e.g. http://localhost:1234/v1). Empty = auto-detect. */
  localEndpoint?: string
  /** Model id to use on the local server. Empty = first available. */
  localModel?: string
}

export interface AudioFingerprint {
  durationSec: number
  sampledSec: number
  bpmEstimate: number | null
  bpmConfidence: 'low' | 'medium' | 'high'
  pitchMedianHz: number | null
  pitchSpreadSemitones: number | null
  voicedRatio: number
  brightness: 'dark' | 'balanced' | 'bright'
  dynamicRangeDb: number
  energyArc: string
  silenceRatio: number
  longestSilenceSec: number
}

export interface MusicCue {
  id: string
  name: string
  sceneRef: string
  intent: string
  genre: string
  mood: string
  tempo: string
  instrumentation: string
  era: string
  structure: string
  vocals: 'instrumental' | 'vocals' | 'either'
  lyricTheme: string
  lyrics: string
  durationSec: number | null
  notes: string
}

export interface VoiceSheet {
  id: string
  name: string
  characterId: string | null
  ageGender: string
  accent: string
  timbre: string
  pitch: string
  pacing: string
  energy: string
  texture: string
  emotionalRange: string
  sampleLine: string
  notes: string
}

export interface ChatMsg {
  role: 'user' | 'assistant'
  text: string
  receipts?: string[]
}

export interface Project {
  id: string
  name: string
  logline: string
  world: string
  defaults: ProjectDefaults
  scenes: Scene[]
  characters: CharacterSheet[]
  artDept: ArtDeptSheet[]
  locations: LocationSheet[]
  lookbook: StyleProfile[]
  references: Reference[]
  mySetups: CustomSetup[]
  music?: MusicCue[]
  voices?: VoiceSheet[]
  copilot?: ChatMsg[]
  createdAt: string
  updatedAt: string
}

export interface ProjectMeta {
  id: string
  name: string
  logline: string
  path: string
  updatedAt: string
  sceneCount: number
  shotCount: number
}

// ---- Brain ----

export type BrainTier = 'fast' | 'standard' | 'top'

/** Which engine powers agent tasks. 'local' talks to any OpenAI-compatible
 *  localhost server (Ollama, LM Studio, vLLM, llama.cpp, KoboldCpp, Jan…). */
export type BrainBackend = 'claude' | 'codex' | 'local'

export interface BrainRequest {
  id: string
  task: string
  system: string
  prompt: string
  images?: string[]
  tier: BrainTier
  expectJson?: boolean
  /** Local backend only: server base URL override and model id. */
  localEndpoint?: string
  localModel?: string
}

export interface BrainResult {
  id: string
  ok: boolean
  text: string
  json?: unknown
  error?: string
  elapsedMs: number
}

export interface BrainStatus {
  claude: { available: boolean; version: string | null }
  codex: { available: boolean; version: string | null }
  local: { available: boolean; version: string | null; endpoint: string | null }
}

export interface LocalModelInfo {
  id: string
}

// ---- IPC surface ----

/** slate-engine HTTP bridge (optional; requires engine serve or auto-start). */
export interface EngineEnsureResult {
  ok: boolean
  message: string
  descriptor: { port: number; token: string; pid?: number } | null
}

export interface EngineHealth {
  engine?: boolean
  comfy?: { ok: boolean; url?: string }
  vision?: { ready?: boolean; model?: string; hint?: string; preferredModel?: string }
  qualityGate?: { passThreshold?: number; maxRetries?: number; judgeModel?: string }
  dryRun?: boolean
  [key: string]: unknown
}

export interface EngineJobStatus {
  active?: boolean
  step?: string
  projectId?: string
  message?: string
  continuitySummary?: string
  lastShotId?: string
  scenePlan?: string
  cancelRequested?: boolean
}

export interface QualityScoresView {
  visualQuality: number
  continuity: number
  artifacts: number
  promptFidelity: number
}

export interface QualityVerdictView {
  accept: boolean
  scores: QualityScoresView
  overall: number
  issues: string[]
  retryHints: string[]
  summary: string
  judgeModel?: string
  mediaPath?: string
}

export interface SlateApi {
  listProjects(): Promise<ProjectMeta[]>
  createProject(name: string): Promise<Project>
  openProject(id: string): Promise<Project | null>
  saveProject(project: Project): Promise<void>
  deleteProject(id: string): Promise<void>
  brainStatus(): Promise<BrainStatus>
  brainRun(req: BrainRequest): Promise<BrainResult>
  brainCancel(id: string): Promise<void>
  brainTest(backend: BrainBackend, local?: { endpoint?: string; model?: string }): Promise<BrainResult>
  localModels(endpoint?: string): Promise<{ endpoint: string | null; models: LocalModelInfo[] }>
  pickMedia(): Promise<string[]>
  pickAudio(): Promise<string[]>
  ingestMedia(projectId: string, path: string): Promise<{ kind: 'image' | 'video'; frames: string[] }>
  stillsDiscover(): Promise<CircledTake[]>
  stillsExtract(projectId: string, mediaPath: string, inSec?: number | null, outSec?: number | null): Promise<string[]>
  analyzeAudio(path: string): Promise<AudioFingerprint>
  pathForFile(file: File): string
  copyText(text: string): Promise<void>
  revealProject(id: string): Promise<void>
  onProjectsChanged(cb: () => void): () => void
  onHelpOpen(cb: () => void): () => void
  onAboutOpen(cb: () => void): () => void
  /** Ensure slate-engine is reachable (start binary if present). */
  engineEnsure(): Promise<EngineEnsureResult>
  engineHealth(): Promise<EngineHealth>
  engineStatus(): Promise<EngineJobStatus>
  engineInvoke(tool: string, args?: Record<string, unknown>): Promise<unknown>
}

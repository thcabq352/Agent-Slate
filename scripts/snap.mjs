// Build README screenshots headlessly: seed a demo project into a temp data
// dir, then run the app in snap mode (hidden window) against a step script.
// Usage: npm run build && node scripts/snap.mjs

import { execFileSync } from 'child_process'
import { mkdirSync, rmSync, writeFileSync } from 'fs'
import { resolve, dirname, join } from 'path'
import { fileURLToPath } from 'url'
import { tmpdir } from 'os'

const root = resolve(dirname(fileURLToPath(import.meta.url)), '..')
const dataDir = join(tmpdir(), 'slate-snap-data')
const outDir = resolve(root, 'docs/images')
rmSync(dataDir, { recursive: true, force: true })
mkdirSync(join(dataDir, 'night-market'), { recursive: true })
mkdirSync(outDir, { recursive: true })

const now = new Date().toISOString()
const shot = (id, name, intent, prompt, spec = {}, extra = {}) => ({
  id, name, intent, prompt,
  spec: { durationSec: 10, fps: 24, aspectRatio: '2.39:1', lens: '35mm', movement: 'track', size: 'MS', angle: 'eye level', ...spec },
  lockedLines: [], mutedLines: [], beatSheet: null, targetModel: 'seedance-2', maxChars: null,
  variants: [], history: [], takes: [], createdAt: now, updatedAt: now, ...extra
})

const project = {
  id: 'night-market', name: 'Night Market',
  logline: 'A courier with a stolen drive has one night to cross a city that wants her caught.',
  world: 'Near-future Kowloon-flavored night city — rain-slick streets, layered neon signage, steam from food stalls, sodium vapor against cyan. Everything wet, everything glowing.',
  defaults: { aspectRatio: '2.39:1', fps: 24, durationSec: 10, targetModel: 'seedance-2', brain: 'cursor' },
  scenes: [
    {
      id: 'sc-rooftop', name: 'Rooftop Chase', synopsis: 'Kaia sprints the rooftops as drones close in; she leaps the alley gap at the market\'s edge.',
      shots: [
        shot('sh-01', 'Shot 01 — Sprint', 'Establish the chase at full speed',
`# Subject
KAIA, 29, wiry frame, cropped black hair, scarred left eyebrow, rain-soaked amber leather jacket, sprints across corrugated rooftop panels, leaping a vent gap mid-stride.

# Composition
Low tracking shot, foreground pipes whipping past, layered neon signage in the background, tight leading room ahead of her run.

# Lighting
Sodium vapor street glow from below, cyan neon rim light, wet surfaces throwing specular highlights, haze in the air.

# Camera
35mm lens, handheld tracking at sprint speed, shallow depth of field, slight motion blur at 1/48 shutter.

# Style
Photoreal, CineStill 800T character, halation on the neon, gritty texture.

# Mood
Desperate, electric urgency — the city itself feels like it's chasing her.`,
          { movement: 'track', size: 'MS' },
          { lockedLines: [2], beatSheet: [
            { from: 0, to: 3, text: 'Kaia enters frame right at full sprint, camera whips to track her' },
            { from: 4, to: 7, text: 'She hurdles the vent gap; drone searchlight rakes across behind her' },
            { from: 8, to: 10, text: 'Camera drops low as she slides under a pipe run toward the roof edge' }
          ] }),
        shot('sh-02', 'Shot 02 — Drone POV', 'The hunter\'s perspective',
`# Subject
Aerial pursuit POV diving between rooftop water tanks, locking onto KAIA's sprinting figure, rain streaking the lens.

# Composition
High angle plunging to low, Kaia small then growing in frame, rooftop geometry layering the dive.

# Lighting
City glow underlighting cloud, neon grid below, searchlight cone sweeping.

# Camera
Drone dive, 24mm, gimbal-stabilized with micro-shake, rolling shutter feel.

# Mood
Predatory, mechanical calm against her panic.`,
          { movement: 'drone', size: 'WS', angle: 'high' }),
        shot('sh-03', 'Shot 03 — The Leap', 'The alley gap — commit or be caught',
`# Subject
KAIA plants a boot on the parapet and launches across the alley gap, arms wide, market lanterns strung below her.

# Composition
Profile wide shot, the gap dead center, both rooftops framing, string lights bisecting the frame under her arc.

# Lighting
Lantern warmth from below meeting cyan moonlight above, silhouette at the apex.

# Camera
85mm from the adjacent roof, locked off, high-speed 96fps ramp at the apex.

# Mood
One suspended breath — triumphant and terrifying.`,
          { movement: 'locked', size: 'WS', lens: '85mm', durationSec: 8 })
      ]
    },
    {
      id: 'sc-market', name: 'Market Ambush',
      synopsis: 'Ground level: Kaia lands into the crowd, and the market erupts as enforcers converge.',
      shots: [
        shot('sh-04', 'Chunk 1 (0:00–0:20)', 'Handoff: she lands hard among stalls, crowd scatters',
`# Subject
KAIA crashes onto a tarp canopy and rolls into the market lane; steam and sparks scatter, vendors recoil.

# Composition
Chaotic handheld in the lane, foreground crowd wiping frame, Kaia rising at center.

# Lighting
Food-stall practicals, red lantern wash, strobing police blue creeping in from the far end.

# Camera
28mm handheld, whip pans between her and the approaching lights.

# Mood
Chaos snapping into focus — the crowd is cover and obstacle at once.`,
          { movement: 'handheld', size: 'MWS', durationSec: 20 })
      ]
    }
  ],
  characters: [
    { id: 'ch-kaia', name: 'KAIA', age: '29', gender: 'woman', ethnicity: 'East Asian', faceFeatures: 'wiry build, sharp jaw, scarred left eyebrow, rain-streaked skin', hair: 'cropped black hair, undercut', clothing: 'rain-soaked amber leather jacket, black courier sling', expression: 'set jaw, eyes scanning', eyeDirection: 'darting to exits', mood: 'cornered but unbroken', environment: 'neon night market', keyLightSide: 'Key light from left', lightingMood: 'Neon practicals', scenario: 'cinematic', notes: '' },
    { id: 'ch-marlow', name: 'MARLOW', age: '61', gender: 'man', ethnicity: 'Black', faceFeatures: 'heavy build, silver stubble, tired knowing eyes', hair: 'silver crop', clothing: 'grey raincoat over dispatch uniform', expression: 'weary calm', eyeDirection: 'steady on the monitors', mood: 'the only calm voice tonight', environment: 'dispatch office', keyLightSide: 'Front key', lightingMood: 'Studio clean', scenario: 'portrait', notes: '' }
  ],
  artDept: [
    { id: 'art-bike', kind: 'vehicle', name: 'Kaia\'s Courier Bike', description: 'Stripped-down café racer, matte black tank with hand-painted koi, mismatched mirrors', materials: 'steel, rubber, brushed alloy', condition: 'scuffed, rain-beaded', era: 'retrofitted 2020s', distinctive: 'koi tank art, LED strip under seat rail', notes: '' }
  ],
  locations: [
    { id: 'loc-market', name: 'The Night Market', interiorExterior: 'exterior', description: 'Two-hundred-meter lane of stalls under strung lanterns and tarps, walled by mid-rise signage', timeOfDay: 'night', weather: 'light rain', architecture: 'dense mid-rise, fire escapes, canopy chaos', textures: 'wet tarps, steam, chrome woks, hand-painted signs', practicalLights: 'lanterns, stall fluorescents, neon signs', notes: '' }
  ],
  lookbook: [
    { id: 'lb-1', source: 'Neon-noir night exteriors', kind: 'custom', tone: 'Romantic dread — beauty in the threat', palette: 'Cyan and teal shadows against sodium amber and lantern red', lighting: 'Practical-driven, hard rims from signage, pools of darkness between', lensLanguage: 'Wide and close or long and compressed — nothing polite in the middle', movement: 'Handheld urgency broken by locked-off tableaux', blocking: 'Figures cutting through layered crowds', editorial: 'Long takes shattered by whip cuts at impacts', notes: 'Let rain carry every light source twice.' }
  ],
  references: [], mySetups: [],
  music: [
    { id: 'cue-chase', name: 'Rooftop Pulse', sceneRef: 'Rooftop Chase', intent: 'Drive the sprint — heartbeat that keeps climbing', genre: 'industrial electronic', mood: 'relentless, cornered', tempo: 'driving ~148 BPM', instrumentation: 'distorted 808 pulse, metallic percussion, detuned synth stabs, sub swells', era: 'analog-modular grit', structure: 'sparse pulse building layer by layer, hard cut on the leap', vocals: 'instrumental', lyricTheme: '', lyrics: '', durationSec: 95, notes: '' }
  ],
  voices: [
    { id: 'v-marlow', name: 'Marlow — Dispatch', characterId: 'ch-marlow', ageGender: 'male, early 60s', accent: 'faded South Side Chicago', timbre: 'low, gravelled warmth', pitch: 'low baritone, narrow range', pacing: 'unhurried, measured beats', energy: 'quiet authority', texture: 'smoke-cured, slight rasp', emotionalRange: 'dry wit to buried worry', sampleLine: 'Kid, I\'ve rerouted you around three checkpoints tonight — don\'t make me watch you waste the fourth.', notes: '' }
  ],
  copilot: [
    { role: 'user', text: 'I need a 90-second rooftop chase for Seedance, 10-second chunks, ending with her leaping into the night market.' },
    { role: 'assistant', text: 'Blocked it out: three hero shots on the rooftop — sprint, drone POV, and the leap at 85mm with a speed ramp — then the market landing opens Scene 2. Continuity carries her amber jacket and the rain throughout; every chunk opens on the previous chunk\'s end state.', receipts: ['✓ Created scene "Rooftop Chase"', '✓ Created "Shot 01 — Sprint" with prompt', '✓ Created "Shot 02 — Drone POV" with prompt', '✓ Created "Shot 03 — The Leap" with prompt', '✓ Created scene "Market Ambush"', '✓ Spotted cue "Rooftop Pulse"'] }
  ],
  createdAt: now, updatedAt: now
}

writeFileSync(join(dataDir, 'night-market', 'project.json'), JSON.stringify(project, null, 2))

// ---- step scripts ----
const wait = (ms) => ({ wait: ms })
const js = (code) => ({ js: code })
const clickText = (sel, text) =>
  js(`[...document.querySelectorAll('${sel}')].find(e=>e.textContent.trim()===${JSON.stringify(text)})?.click()`)
const clickContains = (sel, text) =>
  js(`[...document.querySelectorAll('${sel}')].find(e=>e.textContent.includes(${JSON.stringify(text)}))?.click()`)

const steps = [
  wait(600),
  { shot: '2-home' },
  clickContains('.home-project', 'Night Market'), wait(700),
  { shot: '1-editor' },
  clickText('.tab', 'Coverage'), wait(400),
  { shot: '3-coverage' },
  clickText('.tab', 'Studios'), wait(300),
  clickText('.tab', 'Sound'), wait(400),
  { shot: '4-sound' },
  clickText('.tab', 'Casting'), wait(350),
  js(`[...document.querySelectorAll('.titlebar-side .btn')].find(b=>b.textContent.includes('First AD'))?.click()`),
  wait(600),
  { shot: '5-first-ad' },
  js(`[...document.querySelectorAll('.titlebar-side .btn')].find(b=>b.textContent.includes('First AD'))?.click()`),
  wait(300),
  clickText('.tab', 'Deliver'), wait(500),
  { shot: '6-deliver' }
]

const stepsPath = join(dataDir, 'steps.json')
writeFileSync(stepsPath, JSON.stringify(steps))

console.log('→ running snap (hidden window)…')
execFileSync('npx', ['electron', 'out/main/index.js'], {
  cwd: root,
  stdio: 'inherit',
  env: { ...process.env, SLATE_DATA_DIR: dataDir, SLATE_SNAP_SCRIPT: stepsPath, SLATE_SNAP_OUT: outDir }
})
console.log('✓ screenshots in docs/images/')

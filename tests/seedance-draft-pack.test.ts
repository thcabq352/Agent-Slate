// Pack C reference graphs must match the Partner node contract and stay
// outside workflows/packs so slate-engine cannot queue them.
import { describe, it, expect } from 'vitest'
import { readFileSync, existsSync, readdirSync } from 'fs'
import { join } from 'path'

const root = join(__dirname, '..')
const packDir = join(root, 'workflows/partner/seedance-2-5-draft')
const prompt = readFileSync(join(packDir, 'prompt-template.txt'), 'utf8').trim()

const deskFiles = [
  'api_seedance2_5_draft_t2v.json',
  'api_seedance2_5_draft_i2v.json',
  'api_seedance2_5_draft_r2v.json'
]

const scoutClass: Record<string, string> = {
  'api_seedance2_5_draft_t2v.json': 'ByteDance2TextToVideoNode',
  'api_seedance2_5_draft_i2v.json': 'ByteDance2FirstLastFrameNode',
  'api_seedance2_5_draft_r2v.json': 'ByteDance2ReferenceNodeV2'
}

function load(name: string) {
  return JSON.parse(readFileSync(join(packDir, name), 'utf8'))
}

describe('Seedance 2.5 Draft Pack C', () => {
  it('keeps the Pack C prompt shape', () => {
    expect(prompt).toContain('16:9, cinematic, single continuous take.')
    expect(prompt).toContain('@Image1 character')
    expect(prompt).toContain('@Image2 location')
    expect(prompt).toContain('@Clay Render 1 blocking')
    expect(prompt).toContain('0–10s:')
    expect(prompt).toContain('10–20s:')
    expect(prompt).toContain('20–30s:')
    expect(prompt).toContain('No subtitles, No BGM.')
    expect(prompt).toContain('Keep identity/wardrobe locked.')
    const skill = readFileSync(join(root, 'skills/slate-seedance-draft/references/prompt-pack.md'), 'utf8')
    expect(skill).toContain(prompt)
  })

  it('ships bypassed Desk graphs on the Partner nodes', () => {
    for (const file of deskFiles) {
      const wf = load(file)
      const nodes = wf.nodes as Array<Record<string, unknown>>
      const scout = nodes.find((n) => n.type === scoutClass[file]) as {
        mode: number
        widgets_values_named: Record<string, unknown>
        outputs: Array<{ name: string; links: number[] | null }>
      }
      expect(scout.mode).toBe(4)
      expect(scout.widgets_values_named.model).toBe('Seedance 2.5 Draft')
      expect(scout.widgets_values_named['model.resolution']).toBe('480p')
      expect(scout.widgets_values_named['model.prompt']).toBe(prompt)
      expect(scout.widgets_values_named['model.generate_audio']).toBe(false)
      expect(scout.widgets_values_named['model.duration']).toBe(30)
      expect(scout.widgets_values_named.seed).toBe(0)
      expect(scout.widgets_values_named.control_after_generate).toBe('fixed')
      expect(scout.outputs.map((o) => o.name)).toEqual(['VIDEO', 'draft_task_id'])

      const final = nodes.find((n) => n.type === 'ByteDance2DraftToFinalVideoNode') as {
        mode: number
        inputs: unknown[]
        widgets_values_named: Record<string, unknown>
        title: string
      }
      expect(final.mode).toBe(4)
      expect(final.inputs).toEqual([])
      expect(final.widgets_values_named.draft_task_id).toBe('')
      expect(final.title).toContain('ByteDance Seedance 2.5 Draft to Final Video')
      expect(wf.extra.agentSlate.modelId).toBe('dreamina-seedance-2-5-260628')
      expect(wf.extra.agentSlate.factoryPack).toBe(false)
      expect(wf.extra.agentSlate.agentExecution).toBe('forbidden')
      expect(wf.extra.agentSlate.localOnly).toBe(true)
      const note = nodes.find((n) => n.type === 'MarkdownNote') as {
        widgets_values_named: { text: string }
      }
      expect(note.widgets_values_named.text).toContain('Local-only is a hard requirement')
      expect(note.widgets_values_named.text).not.toContain('queue once')
    }
  })

  it('locks 16:9 on T2V and R2V and leaves I2V ratio to the first frame', () => {
    const t2v = load('api_seedance2_5_draft_t2v.json')
    const r2v = load('api_seedance2_5_draft_r2v.json')
    const i2v = load('api_seedance2_5_draft_i2v.json')
    const named = (wf: { nodes: Array<{ type: string; widgets_values_named?: Record<string, unknown> }> }, type: string) =>
      wf.nodes.find((n) => n.type === type)!.widgets_values_named!
    expect(named(t2v, 'ByteDance2TextToVideoNode')['model.ratio']).toBe('16:9')
    expect(named(r2v, 'ByteDance2ReferenceNodeV2')['model.ratio']).toBe('16:9')
    expect(named(r2v, 'ByteDance2ReferenceNodeV2')['model.task_type']).toBe('reference')
    expect(named(i2v, 'ByteDance2FirstLastFrameNode')['model.ratio']).toBeUndefined()
    const i2vNode = i2v.nodes.find((n: { type: string }) => n.type === 'ByteDance2FirstLastFrameNode')
    expect(i2vNode.inputs.find((i: { name: string }) => i.name === 'last_frame').link).toBeNull()
  })

  it('does not wire draft_task_id into the promote node', () => {
    for (const file of deskFiles) {
      const wf = load(file)
      const links = wf.links as Array<[number, number, number, number, number, string]>
      const final = wf.nodes.find((n: { type: string }) => n.type === 'ByteDance2DraftToFinalVideoNode')
      const intoFinal = links.filter((l) => l[3] === final.id)
      expect(intoFinal).toEqual([])
      const fromScoutId = links.filter((l) => l[5] === 'STRING')
      expect(fromScoutId).toHaveLength(1)
      const save = wf.nodes.find((n: { id: number }) => n.id === fromScoutId[0][3])
      expect(save.type).toBe('SaveText')
    }
  })

  it('keeps API maps off the factory pack directory', () => {
    expect(existsSync(join(packDir, 'manifest.json'))).toBe(false)
    expect(existsSync(join(packDir, 'workflow.api.json'))).toBe(false)
    const packs = readdirSync(join(root, 'workflows/packs'))
    expect(packs.some((name) => name.toLowerCase().includes('seedance'))).toBe(false)

    const contract = load('contract.json')
    expect(contract.modelId).toBe('dreamina-seedance-2-5-260628')
    expect(contract.factoryPack).toBe(false)
    expect(contract.queueFromThisRepo).toBe(false)
    expect(contract.officialTemplateIds).toEqual({
      t2v: 'api_seedance2_5_draft_t2v',
      i2v: 'api_seedance2_5_draft_i2v',
      r2v: 'api_seedance2_5_draft_r2v'
    })
    expect(contract.nodes.promote.displayName).toBe('ByteDance Seedance 2.5 Draft to Final Video')

    const promote = load('api_seedance2_5_draft_promote.api.json')
    expect(Object.keys(promote).sort()).toEqual(['4', '5'])
    expect(promote['4'].class_type).toBe('ByteDance2DraftToFinalVideoNode')
    expect(promote['4'].inputs.draft_task_id).toBe('')
    const t2vApi = load('api_seedance2_5_draft_t2v.api.json')
    expect(t2vApi['1'].inputs.model).toBe('Seedance 2.5 Draft')
    expect(t2vApi['1'].inputs['model.resolution']).toBe('480p')
    expect(t2vApi['1'].inputs.control_after_generate).toBeUndefined()
    expect(t2vApi['4']).toBeUndefined()
  })

  it('enforces local-only Pack C routing', () => {
    const agents = JSON.parse(readFileSync(join(root, 'AGENTS.json'), 'utf8'))
    expect(agents.factory.localOnly.hardRequirement).toBe(true)
    expect(agents.factory.localOnly.cloudApis).toBe(false)
    expect(agents.factory.localOnly.cloudServices).toBe(false)
    expect(agents.factory.localOnly.hostedInference).toBe(false)
    expect(agents.factory.localOnly.packC.execution).toBe('local-comfy')
    expect(agents.factory.localOnly.packC.comfy).toBe('http://127.0.0.1:8188')
    expect(agents.factory.localOnly.packC.partnerGraphsExecutable).toBe(false)
    const seedance = agents.whenUserSays.find(
      (row: { intent: string }) => row.intent === 'Seedance 2.5 one-take / Draft to Final'
    )
    expect(seedance.do).toContain('Local-only hard requirement')
    expect(seedance.do).toContain('http://127.0.0.1:8188')
    expect(seedance.do).not.toContain('Comfy Desk only')

    const skill = readFileSync(join(root, 'skills/slate-seedance-draft/SKILL.md'), 'utf8')
    expect(skill).toContain('Local-only is a hard requirement')
    expect(skill).toContain('http://127.0.0.1:8188')
    expect(skill).toContain('default-still')
    expect(skill).not.toContain('choose to spend')
    expect(skill).not.toContain('Enable Stage 1')

    const contract = load('contract.json')
    expect(contract.localOnly.hardRequirement).toBe(true)
    expect(contract.localOnly.agentExecution).toBe('forbidden')
    expect(contract.localOnly.cloudApis).toBe(false)
    expect(contract.localOnly.hostedInference).toBe(false)
    expect(contract.localOnly.localComfy).toBe('http://127.0.0.1:8188')
  })
})

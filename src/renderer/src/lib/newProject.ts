import type { Project } from '../../../shared/types'

export function newProjectShape(name: string): Project {
  const now = new Date().toISOString()
  return {
    id: `p-${Date.now().toString(36)}-${Math.random().toString(36).slice(2, 8)}`,
    name,
    logline: '',
    world: '',
    defaults: {
      aspectRatio: '16:9',
      fps: 24,
      durationSec: 8,
      targetModel: 'seedance-2',
      brain: 'cursor',
      localEndpoint: '',
      localModel: ''
    },
    scenes: [],
    characters: [],
    artDept: [],
    locations: [],
    lookbook: [],
    references: [],
    mySetups: [],
    music: [],
    voices: [],
    createdAt: now,
    updatedAt: now
  }
}

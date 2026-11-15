import { describe, it, expect } from 'vitest'
import { projectWebMap } from './webProjection'
describe('web GPS projection', () => {
  it('uses the map projector and normalizes its real viewport, keeping hidden/offscreen/occluded points out', () => {
    const request = {
      id: 7,
      points: [
        { marker: 1, x: 12, z: 34, distance: 50 },
        { marker: 2, x: 200, z: 3, distance: 4 },
        { marker: 3, x: 100, z: 10, distance: 100 },
      ],
    }
    const view = {
      rect: { left: 200, top: 100, width: 600, height: 400 },
      width: 1200,
      height: 800,
      frameWidth: 1000,
      frameHeight: 600,
      project: (x: number, z: number) => ({ x: x * 10, y: z * 5 }),
      occluded: (x: number) => x > 600,
    }
    const out = projectWebMap(request, view)
    expect(out.clip).toEqual([0.2, 1 / 6, 0.6, 2 / 3])
    expect(out.markers).toHaveLength(1)
    expect(out.markers[0]).toEqual({ marker: 1, x: 0.26, y: 0.30833333333333335, distance: 50 })
    expect(projectWebMap(request, null).markers).toEqual([])
  })
  it('does not fabricate points for missing maps or out of bounds projections', () => {
    const request = { id: 2, points: [{ marker: 1, x: 1, z: 1, distance: 20 }] }
    expect(projectWebMap(request, null)).toEqual({ id: 2, clip: null, markers: [] })
  })
})

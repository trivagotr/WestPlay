import type { WebMapProjection, WebMapRequest } from '../../shared/webMap'
export interface ProjectionView {
  rect: { left: number; top: number; width: number; height: number }
  width: number
  height: number
  frameWidth: number
  frameHeight: number
  project: (x: number, z: number) => { x: number; y: number }
  occluded: (x: number, y: number) => boolean
}
/** Uses the actual map camera's projector, including bearing/pitch/pan/zoom. Never mutates the map. */
export function projectWebMap(request: WebMapRequest, view: ProjectionView | null): WebMapProjection {
  const reply: WebMapProjection = { id: request.id, clip: null, markers: [] }
  if (!view || request.points.length > 128) return reply
  const { rect, width, height, frameWidth, frameHeight } = view
  if (
    ![rect.left, rect.top, rect.width, rect.height, width, height, frameWidth, frameHeight].every(Number.isFinite) ||
    Math.min(rect.width, rect.height, width, height, frameWidth, frameHeight) <= 0
  )
    return reply
  const left = Math.max(0, rect.left),
    top = Math.max(0, rect.top),
    right = Math.min(frameWidth, rect.left + rect.width),
    bottom = Math.min(frameHeight, rect.top + rect.height)
  if (right <= left || bottom <= top) return reply
  reply.clip = [left / frameWidth, top / frameHeight, (right - left) / frameWidth, (bottom - top) / frameHeight]
  for (const point of request.points) {
    if (
      !Number.isInteger(point.marker) ||
      point.marker < 1 ||
      point.marker > 128 ||
      ![point.x, point.z, point.distance].every(Number.isFinite) ||
      point.distance < 0
    )
      continue
    const at = view.project(point.x, point.z)
    const x = rect.left + (at.x * rect.width) / width,
      y = rect.top + (at.y * rect.height) / height
    if (
      !Number.isFinite(x) ||
      !Number.isFinite(y) ||
      x < left + 8 ||
      x > right - 8 ||
      y < top + 8 ||
      y > bottom - 8 ||
      view.occluded(x, y)
    )
      continue
    reply.markers.push({ marker: point.marker, x: x / frameWidth, y: y / frameHeight, distance: point.distance })
  }
  return reply
}

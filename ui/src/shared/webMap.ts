/** Private PC-side world points; only normalized screen markers reach the browser. */
export interface WebMapPoint {
  marker: number
  x: number
  z: number
  distance: number
}
export interface WebMapRequest {
  id: number
  points: WebMapPoint[]
}
export interface WebMapMarker {
  marker: number
  x: number
  y: number
  distance: number
}
export interface WebMapProjection {
  id: number
  clip: [number, number, number, number] | null
  markers: WebMapMarker[]
}

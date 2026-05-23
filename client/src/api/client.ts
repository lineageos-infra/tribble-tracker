import type { FilterColumn, StatsResponse } from './types'

async function request<T>(url: string): Promise<T> {
  const res = await fetch(url)
  if (!res.ok) {
    throw new Error(`Request failed: ${res.status} ${res.statusText}`)
  }
  return (await res.json()) as T
}

export function getStats(): Promise<StatsResponse> {
  return request<StatsResponse>('/api/v1/stats')
}

export function getFilteredStats(column: FilterColumn, name: string): Promise<StatsResponse> {
  return request<StatsResponse>(`/api/v1/stats/${column}/${encodeURIComponent(name)}`)
}

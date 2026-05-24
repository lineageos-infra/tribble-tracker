// SPDX-FileCopyrightText: 2026 The LineageOS Project
//
// SPDX-License-Identifier: Apache-2.0

import type { ActiveFilter } from '@/utils/filters'
import type { StatsResponse } from './types'

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

export function getFilteredStats(filters: ActiveFilter[]): Promise<StatsResponse> {
  if (!filters.length) return getStats()

  const query = new URLSearchParams()
  for (const filter of filters) {
    query.set(filter.column, filter.name)
  }

  const url = `/api/v1/stats/filter?${query.toString()}`
  return request<StatsResponse>(url)
}

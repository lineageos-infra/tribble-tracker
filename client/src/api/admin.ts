// SPDX-FileCopyrightText: 2026 The LineageOS Project
//
// SPDX-License-Identifier: Apache-2.0

export interface BannedItem {
  version: string | null
  model: string | null
  note: string | null
  affected_installations: number | null
}

export interface TotalInstallationsItem {
  model: string
  version_raw: string
  asn: number
  installations: number
}

export interface TopAsnItem {
  asn: number
  asn_owner: string
  devices: number
  top_model: string
  top_model_count: number
}

export interface InstallationFilters {
  model?: string
  country?: string
  version?: string
  carrier?: string
  asn?: string
}

async function request<T>(url: string, method?: string, body?: unknown): Promise<T> {
  const res = await fetch(url, {
    method: method ?? 'GET',
    headers: body === undefined ? undefined : { 'Content-Type': 'application/json' },
    body: body === undefined ? undefined : JSON.stringify(body)
  })
  if (!res.ok) {
    throw new Error(`Request failed: ${res.status} ${res.statusText}`)
  }
  const isJson = res.headers.get('content-type')?.includes('application/json')
  return (isJson ? await res.json() : await res.text()) as T
}

export function listBans(): Promise<BannedItem[]> {
  return request<BannedItem[]>('/internal/ban/list')
}

export function reapBans(): Promise<number> {
  return request<number>('/internal/ban/reap', 'POST')
}

export function banModels(models: string[], note?: string): Promise<string> {
  return request('/internal/ban/models', 'POST', { models, note })
}

export function unbanModels(models: string[]): Promise<string> {
  return request('/internal/ban/models', 'DELETE', { models })
}

export function banVersions(versions: string[], note?: string): Promise<string> {
  return request('/internal/ban/versions', 'POST', { versions, note })
}

export function unbanVersions(versions: string[]): Promise<string> {
  return request('/internal/ban/versions', 'DELETE', { versions })
}

export function getTopAsns(): Promise<TopAsnItem[]> {
  return request<TopAsnItem[]>('/internal/asn')
}

export function getInstallations(filters: InstallationFilters): Promise<TotalInstallationsItem[]> {
  const query = new URLSearchParams()
  for (const [key, value] of Object.entries(filters)) {
    if (value) query.set(key, value)
  }

  const suffix = query.toString()
  const url = suffix ? `/internal/installations?${suffix}` : '/internal/installations'
  return request<TotalInstallationsItem[]>(url)
}

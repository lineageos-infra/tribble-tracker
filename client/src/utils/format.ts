// SPDX-FileCopyrightText: 2026 The LineageOS Project
//
// SPDX-License-Identifier: Apache-2.0

import type { FilterColumn } from '@/api/types'

export function formatNumber(n: number, compact = false): string {
  const options: Intl.NumberFormatOptions = compact ? { notation: 'compact' } : {}
  return new Intl.NumberFormat('en-US', options).format(n)
}

const regionDisplay = (() => {
  return new Intl.DisplayNames(['en'], { type: 'region' })
})()

export function countryName(code: string): string | null {
  if (!code || code === 'Unknown' || code.length !== 2) return null
  try {
    return regionDisplay.of(code.toUpperCase()) ?? null
  } catch {
    return null
  }
}

export function countryFlag(code: string): string | null {
  if (!/^[A-Za-z]{2}$/.test(code)) return null
  return String.fromCodePoint(
    ...[...code.toUpperCase()].map((c) => 0x1f1e6 + (c.charCodeAt(0) - 'A'.charCodeAt(0)))
  )
}

export function formatColumnLabel(column: FilterColumn): string {
  switch (column) {
    case 'model':
      return 'Device'
    case 'country':
      return 'Country'
    case 'version':
      return 'Version'
    case 'carrier':
      return 'Carrier'
    default:
      return column
  }
}

export function formatFilterValue(column: string, name: string): string {
  if (column === 'country') {
    const flag = countryFlag(name)
    const full = countryName(name)
    if (flag && full) return `${flag} ${full}`
    if (full) return full
  }
  return name
}

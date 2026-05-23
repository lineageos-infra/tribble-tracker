const numberFormatter = new Intl.NumberFormat('en-US')

export function formatNumber(n: number): string {
  return numberFormatter.format(n)
}

const regionDisplay = (() => {
  try {
    return new Intl.DisplayNames(['en'], { type: 'region' })
  } catch {
    return null
  }
})()

export function countryName(code: string): string | null {
  if (!code || code === 'Unknown' || code.length !== 2) return null
  try {
    return regionDisplay?.of(code.toUpperCase()) ?? null
  } catch {
    return null
  }
}

export function countryFlag(code: string): string | null {
  if (!code || code === 'Unknown' || code.length !== 2) return null
  const upper = code.toUpperCase()
  if (!/^[A-Z]{2}$/.test(upper)) return null
  return String.fromCodePoint(
    ...[...upper].map((c) => 0x1f1e6 + (c.charCodeAt(0) - 'A'.charCodeAt(0)))
  )
}

export function formatColumnLabel(column: string): string {
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

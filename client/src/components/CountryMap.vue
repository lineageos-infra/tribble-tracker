<!--
SPDX-FileCopyrightText: 2026 The LineageOS Project

SPDX-License-Identifier: Apache-2.0
-->

<script setup lang="ts">
import { COUNTRY_LAT_LONG } from '@/data/countryLatLong'
import { countryFlag, countryName, formatNumber } from '@/utils/format'
import { VisLeafletMap } from '@unovis/vue'
import { useMediaQuery, usePreferredDark } from '@vueuse/core'
import { computed, ref } from 'vue'

interface Point {
  id: string
  code: string
  label: string
  count: number
  latitude: number
  longitude: number
}

const props = defineProps<{
  entries: Record<string, number>
  total: number
}>()

const points = computed<Point[]>(() => {
  const out: Point[] = []
  for (const [code, count] of Object.entries(props.entries)) {
    const centroid = COUNTRY_LAT_LONG[code.toUpperCase()]
    if (!centroid) continue
    out.push({
      id: code,
      code,
      label: countryName(code) ?? code,
      count,
      latitude: centroid[0],
      longitude: centroid[1]
    })
  }
  return out
})

const pointByCode = computed(() => {
  const m = new Map<string, Point>()
  for (const p of points.value) m.set(p.code, p)
  return m
})

const maxCount = computed(() => points.value.reduce((m, p) => Math.max(m, p.count), 1))

const pointRadius = (d: Point) => {
  const minR = 5
  const maxR = 36
  const t = Math.sqrt(d.count) / Math.sqrt(maxCount.value)
  return minR + (maxR - minR) * t
}

const isDark = usePreferredDark()
const isSmUp = useMediaQuery('(min-width: 640px)')

// unovis VisTooltip doesn't wire into the HTML-based LeafletMap, so delegate
// mouse events on the rendered marker paths and render our own tooltip.
const mapWrapper = ref<HTMLElement | null>(null)
const tooltipPoint = ref<Point | null>(null)
const tooltipPos = ref({ x: 0, y: 0 })

const tooltipPct = computed(() =>
  tooltipPoint.value ? ((tooltipPoint.value.count / props.total) * 100).toFixed(2) : ''
)

function findPointCode(target: EventTarget | null): string | null {
  let el = target as Element | null
  while (el && el !== mapWrapper.value) {
    const id = el.id || ''
    if (id.startsWith('point-')) return id.slice('point-'.length)
    if (id.startsWith('label-')) return id.slice('label-'.length)
    el = el.parentElement
  }
  return null
}

function handleMove(e: MouseEvent) {
  const code = findPointCode(e.target)
  if (!code) {
    tooltipPoint.value = null
    return
  }
  const point = pointByCode.value.get(code)
  if (!point) {
    tooltipPoint.value = null
    return
  }
  const wrapRect = mapWrapper.value?.getBoundingClientRect()
  if (!wrapRect) return
  tooltipPoint.value = point
  tooltipPos.value = {
    x: e.clientX - wrapRect.left,
    y: e.clientY - wrapRect.top
  }
}

function handleLeave() {
  tooltipPoint.value = null
}
</script>

<template>
  <section class="flex flex-col gap-3">
    <header class="flex flex-wrap items-baseline justify-between gap-2 px-2">
      <div>
        <h2 class="text-lg font-medium text-on-surface">Where installs come from</h2>
        <p class="text-xs text-on-surface-muted">
          {{ formatNumber(points.length) }} countries · {{ formatNumber(total) }} total installs
        </p>
      </div>
    </header>
    <div
      ref="mapWrapper"
      class="map-container relative w-full overflow-hidden rounded-3xl"
      @mousemove="handleMove"
      @mouseleave="handleLeave"
    >
      <VisLeafletMap
        :key="isDark ? 'dark' : 'light'"
        :height="isSmUp ? 520 : 460"
        :data="points"
        :style="
          isDark
            ? 'https://tiles.openfreemap.org/styles/dark'
            : 'https://tiles.openfreemap.org/styles/positron'
        "
        :point-radius="pointRadius"
        :point-color="isDark ? 'rgba(204, 232, 233, 0.65)' : 'rgba(22, 124, 128, 0.65)'"
        :point-label="(d: Point) => formatNumber(d.count)"
        :cluster-color="isDark ? 'rgba(204, 232, 233, 0.75)' : 'rgba(22, 124, 128, 0.75)'"
        :fit-view-padding="[40, 40]"
        :attribution="[
          `<a href=&quot;https://openfreemap.org&quot; target=&quot;_blank&quot;>OpenFreeMap</a> | <a href=&quot;https://www.openmaptiles.org/&quot; target=&quot;_blank&quot;>© OpenMapTiles</a> | <a href=&quot;https://www.openstreetmap.org/copyright&quot; target=&quot;_blank&quot;>Data from OpenStreetMap</a>`
        ]"
      />
      <div
        v-if="tooltipPoint"
        class="pointer-events-none absolute z-10 -translate-x-1/2 -translate-y-full rounded-md border border-outline-variant bg-surface-elevated px-3 py-2 text-xs whitespace-nowrap text-on-surface shadow-md"
        :style="{ left: tooltipPos.x + 'px', top: tooltipPos.y - 12 + 'px' }"
      >
        <div class="flex items-center gap-1.5 font-medium">
          <span v-if="tooltipPoint.code !== 'Unknown'">{{ countryFlag(tooltipPoint.code) }}</span>
          <span>{{ tooltipPoint.label }}</span>
        </div>
        <div class="mt-0.5 text-on-surface-muted">
          {{ formatNumber(tooltipPoint.count) }} installs · {{ tooltipPct }}%
        </div>
      </div>
    </div>
  </section>
</template>

<style scoped>
.map-container {
  --vis-map-container-background-color: transparent;
  --vis-dark-map-container-background-color: transparent;
}
</style>

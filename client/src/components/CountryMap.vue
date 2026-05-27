<!--
SPDX-FileCopyrightText: 2026 The LineageOS Project

SPDX-License-Identifier: Apache-2.0
-->

<script setup lang="ts">
import { COUNTRY_LAT_LONG } from '@/data/countryLatLong'
import { countryFlag, countryName, formatNumber } from '@/utils/format'
import { TopoJSONMap, type TopoJSONMapPoint } from '@unovis/ts'
import { WorldMap110mAlphaTopoJSON } from '@unovis/ts/maps'
import { VisSingleContainer, VisTooltip, VisTopoJSONMap } from '@unovis/vue'
import { computed } from 'vue'

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

const points = computed<Point[]>(() =>
  Object.entries(props.entries).flatMap(([code, count]) => {
    const centroid = COUNTRY_LAT_LONG[code.toUpperCase()]
    if (!centroid) return []
    return [
      {
        id: code,
        code,
        label: countryName(code) ?? code,
        count,
        latitude: centroid[0],
        longitude: centroid[1]
      }
    ]
  })
)

const maxCount = computed(() => points.value.reduce((m, p) => Math.max(m, p.count), 1))
const radiusForCount = (count: number) => {
  const t = Math.min(Math.pow(count / maxCount.value, 0.3), 1)
  return 8 + 28 * t
}

const sumClusterCount = (d: { clusterPoints?: Point[] }) =>
  (d.clusterPoints ?? []).reduce((s, p) => s + p.count, 0)
const clusterRadius = (d: { clusterPoints?: Point[] }) => radiusForCount(sumClusterCount(d))
const clusterLabel = (d: { clusterPoints?: Point[] }) => formatNumber(sumClusterCount(d), true)

const renderTooltip = (label: string, count: number, flag = '') => {
  const pct = ((count / props.total) * 100).toFixed(2)
  return `
    <div class="rounded-md border border-outline-variant bg-surface-elevated px-3 py-2 text-xs text-on-surface shadow-md">
      <div class="flex items-center gap-1.5 font-medium">${flag ? flag + ' ' : ''}${label}</div>
      <div class="mt-0.5 text-on-surface-muted">${formatNumber(count)} installs · ${pct}%</div>
    </div>
  `
}

const pointTooltip = (d: TopoJSONMapPoint<Point>) => {
  if (d.isCluster) {
    const cluster = d.properties as { clusterPoints?: Point[]; pointCount?: number }
    const label = `${cluster.pointCount ?? cluster.clusterPoints?.length ?? 0} countries`
    return renderTooltip(label, sumClusterCount(cluster))
  }
  const point = d.properties as Point
  const flag = point.code !== 'Unknown' ? (countryFlag(point.code) ?? '') : ''
  return renderTooltip(point.label, point.count, flag)
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
    <VisSingleContainer
      :data="{ points }"
      :height="520"
      class="vis-country-map relative w-full overflow-hidden rounded-3xl bg-surface-elevated"
      @wheel.prevent
    >
      <VisTopoJSONMap
        point-color="var(--color-bar-fill)"
        cluster-color="var(--color-bar-fill)"
        :topojson="WorldMap110mAlphaTopoJSON"
        :point-radius="(d: Point) => radiusForCount(d.count)"
        :point-label="(d: Point) => formatNumber(d.count, true)"
        :cluster-radius="clusterRadius"
        :cluster-label="clusterLabel"
        :clustering="true"
        :cluster-expand-on-click="false"
      />
      <VisTooltip :triggers="{ [TopoJSONMap.selectors.point]: pointTooltip }" />
    </VisSingleContainer>
  </section>
</template>

<style scoped>
.vis-country-map {
  --vis-font-family: var(--font-sans);
  --vis-map-feature-color: #d4e4e4;
  --vis-map-boundary-color: #eaf2f2;
  --vis-tooltip-background-color: transparent;
  --vis-tooltip-border-color: transparent;
}

@media (prefers-color-scheme: dark) {
  .vis-country-map {
    --vis-map-feature-color: #2a3838;
    --vis-map-boundary-color: #131a1a;
  }
}

.vis-country-map :deep(text[class*='-label']) {
  font-size: calc(var(--vis-map-point-label-font-size) / var(--vis-map-current-zoom-level, 1));
}
</style>

<!--
SPDX-FileCopyrightText: 2026 The LineageOS Project

SPDX-License-Identifier: Apache-2.0
-->

<script setup lang="ts">
import FilterChip from '@/components/FilterChip.vue'
import StatCard from '@/components/StatCard.vue'
import { useStats } from '@/composables/useStats'
import type { FilterColumn } from '@/api/types'
import { filtersFromRoute, routeForClearingFilter } from '@/utils/filters'
import { formatNumber } from '@/utils/format'
import { computed } from 'vue'
import { useRoute } from 'vue-router'

const CARDS: ReadonlyArray<{ column: FilterColumn; title: string }> = [
  { column: 'model', title: 'Top Devices' },
  { column: 'country', title: 'Top Countries' },
  { column: 'version', title: 'Top Versions' },
  { column: 'carrier', title: 'Top Carriers' },
]

const GRID_COLS = [
  '',
  'md:grid-cols-1',
  'md:grid-cols-2',
  'md:grid-cols-2 xl:grid-cols-3',
  'md:grid-cols-2 xl:grid-cols-4',
]

const route = useRoute()
const filters = computed(() => filtersFromRoute(route))
const { data, loading, error } = useStats(() => filters.value.length ? filters.value : null)

const visibleCards = computed(() => {
  if (!data.value) return []
  const filtered = new Set(filters.value.map((f) => f.column))
  return CARDS
    .filter(({ column }) => !filtered.has(column) && Object.keys(data.value![column]).length)
    .map(({ column, title }) => ({ column, title, entries: data.value![column] }))
})
</script>

<template>
  <div class="mx-auto flex max-w-7xl flex-col gap-6">
    <header class="flex flex-col gap-3 px-1">
      <div v-if="filters.length" class="flex flex-wrap gap-2">
        <FilterChip
          v-for="filter in filters"
          :key="`${filter.column}:${filter.name}`"
          :column="filter.column"
          :name="filter.name"
          :clear-to="routeForClearingFilter(route, filter)"
        />
      </div>
      <div class="flex items-baseline gap-3">
        <span class="text-4xl font-medium tabular-nums text-on-surface sm:text-5xl">
          {{ formatNumber(data?.total ?? 0) }}
        </span>
        <span class="text-sm text-on-surface-muted">matching installs</span>
      </div>
    </header>

    <p
      v-if="error"
      class="rounded-2xl bg-surface-elevated p-6 text-center text-sm text-on-surface-muted"
    >
      Couldn&rsquo;t load statistics. {{ error.message }}
    </p>

    <p
      v-else-if="loading && !data"
      class="rounded-2xl bg-surface-elevated p-6 text-center text-sm text-on-surface-muted"
    >
      Loading…
    </p>

    <div v-else-if="data" class="grid gap-4" :class="GRID_COLS[visibleCards.length]">
      <StatCard
        v-for="card in visibleCards"
        :key="card.column"
        :title="card.title"
        :column="card.column"
        :entries="card.entries"
      />
    </div>
  </div>
</template>

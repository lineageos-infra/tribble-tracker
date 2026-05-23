<!--
SPDX-FileCopyrightText: 2026 The LineageOS Project

SPDX-License-Identifier: Apache-2.0
-->

<script setup lang="ts">
import FilterChip from '@/components/FilterChip.vue'
import StatCard from '@/components/StatCard.vue'
import { useStats } from '@/composables/useStats'
import { filtersFromRoute, routeForClearingFilter } from '@/utils/filters'
import { formatNumber } from '@/utils/format'
import { computed } from 'vue'
import { useRoute } from 'vue-router'

const route = useRoute()
const filters = computed(() => filtersFromRoute(route))
const primaryFilter = computed(() => filters.value[0])

const { data, loading, error } = useStats(() => {
  if (!filters.value.length) return null
  return filters.value
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

    <div v-else-if="data" class="grid gap-4 md:grid-cols-2 xl:grid-cols-3">
      <StatCard
        v-if="primaryFilter.column !== 'model' && Object.keys(data.model).length"
        title="Top Devices"
        column="model"
        :entries="data.model"
      />
      <StatCard
        v-if="primaryFilter.column !== 'country' && Object.keys(data.country).length"
        title="Top Countries"
        column="country"
        :entries="data.country"
      />
      <StatCard
        v-if="primaryFilter.column !== 'version' && Object.keys(data.version).length"
        title="Top Versions"
        column="version"
        :entries="data.version"
      />
      <StatCard
        v-if="primaryFilter.column !== 'carrier' && data.carrier && Object.keys(data.carrier).length"
        title="Top Carriers"
        column="carrier"
        :entries="data.carrier"
      />
    </div>
  </div>
</template>

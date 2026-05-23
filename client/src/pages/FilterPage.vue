<script setup lang="ts">
import { isFilterColumn, type FilterColumn } from '@/api/types'
import FilterChip from '@/components/FilterChip.vue'
import StatCard from '@/components/StatCard.vue'
import { useStats } from '@/composables/useStats'
import { formatNumber } from '@/utils/format'
import { computed } from 'vue'
import { useRoute } from 'vue-router'

const route = useRoute()

const column = computed<FilterColumn | null>(() => {
  const c = route.params.column as string
  return isFilterColumn(c) ? c : null
})
const name = computed(() => route.params.name as string)

const { data, loading, error } = useStats(() => {
  if (!column.value || !name.value) return null
  return { column: column.value, name: name.value }
})
</script>

<template>
  <div class="mx-auto flex max-w-7xl flex-col gap-6">
    <header class="flex flex-col gap-3 px-1">
      <FilterChip v-if="column" :column="column" :name="name" />
      <div class="flex items-baseline gap-3">
        <span class="text-4xl font-medium tabular-nums text-on-surface sm:text-5xl">
          {{ formatNumber(data?.total ?? 0) }}
        </span>
        <span class="text-sm text-on-surface-muted">matching installs</span>
      </div>
    </header>

    <p
      v-if="error"
      class="rounded-2xl border border-outline-variant bg-surface-elevated p-6 text-center text-sm text-on-surface-muted"
    >
      Couldn&rsquo;t load statistics. {{ error.message }}
    </p>

    <p
      v-else-if="loading && !data"
      class="rounded-2xl border border-outline-variant bg-surface-elevated p-6 text-center text-sm text-on-surface-muted"
    >
      Loading…
    </p>

    <div v-else-if="data" class="grid gap-4 md:grid-cols-2 xl:grid-cols-4">
      <StatCard
        v-if="column !== 'model' && Object.keys(data.model).length"
        title="Top Devices"
        column="model"
        :entries="data.model"
      />
      <StatCard
        v-if="column !== 'country' && Object.keys(data.country).length"
        title="Top Countries"
        column="country"
        :entries="data.country"
      />
      <StatCard
        v-if="column !== 'version' && Object.keys(data.version).length"
        title="Top Versions"
        column="version"
        :entries="data.version"
      />
      <StatCard
        v-if="column !== 'carrier' && data.carrier && Object.keys(data.carrier).length"
        title="Top Carriers"
        column="carrier"
        :entries="data.carrier"
      />
    </div>
  </div>
</template>

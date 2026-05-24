<!--
SPDX-FileCopyrightText: 2026 The LineageOS Project

SPDX-License-Identifier: Apache-2.0
-->

<script setup lang="ts">
import type { FilterColumn } from '@/api/types'
import { countryName, formatNumber } from '@/utils/format'
import { Search } from '@lucide/vue'
import { computed, ref } from 'vue'
import StatBarRow from './StatBarRow.vue'

const props = defineProps<{
  title: string
  column: FilterColumn
  entries: Record<string, number>
}>()

const query = ref('')

// Re-sort by count desc: JS iterates integer-like string keys first, which
// otherwise hoists purely-numeric carrier codes above higher-count entries.
const list = computed(() => Object.entries(props.entries).sort((a, b) => b[1] - a[1]))

const max = computed(() => list.value.reduce((m, [, c]) => Math.max(m, c), 0))

const filtered = computed(() => {
  const q = query.value.trim().toLowerCase()
  if (!q) return list.value
  return list.value.filter(([k]) => {
    if (k.toLowerCase().includes(q)) return true
    if (props.column === 'country') {
      const name = countryName(k)
      if (name && name.toLowerCase().includes(q)) return true
    }
    return false
  })
})
</script>

<template>
  <section class="flex h-full flex-col rounded-3xl bg-surface-elevated p-4 sm:p-5">
    <header class="mb-3 flex items-baseline justify-between gap-2 px-2">
      <h2 class="text-lg font-medium text-on-surface">{{ title }}</h2>
      <span class="text-xs text-on-surface-muted">
        {{ formatNumber(list.length) }}
      </span>
    </header>

    <label class="relative mb-3 block px-1">
      <span class="sr-only">Filter {{ title }}</span>
      <Search
        class="pointer-events-none absolute top-1/2 left-4 size-4 -translate-y-1/2 text-on-surface-muted"
        aria-hidden="true"
      />
      <input
        v-model="query"
        type="search"
        :placeholder="`Search ${title.toLowerCase()}`"
        class="w-full rounded-full border border-outline-variant bg-surface py-2 pr-3 pl-9 text-sm text-on-surface placeholder:text-on-surface-muted focus:border-brand-primary focus:outline-none"
      />
    </label>

    <div class="-mx-1 flex-1 overflow-y-auto pr-1" style="max-height: 480px">
      <ol v-if="filtered.length" class="space-y-0.5">
        <li v-for="([name, count], i) in filtered" :key="name">
          <StatBarRow :rank="i + 1" :column="column" :name="name" :count="count" :max="max" />
        </li>
      </ol>
      <p v-else-if="query.length" class="px-3 py-8 text-center text-sm text-on-surface-muted">
        No matches for &ldquo;{{ query }}&rdquo;
      </p>
      <p v-else-if="!list.length" class="px-3 py-8 text-center text-sm text-on-surface-muted">
        No data available
      </p>
    </div>
  </section>
</template>

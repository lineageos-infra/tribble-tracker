<!--
SPDX-FileCopyrightText: 2026 The LineageOS Project

SPDX-License-Identifier: Apache-2.0
-->

<script setup lang="ts">
import { FILTER_COLUMNS } from '@/api/types'
import CountryMap from '@/components/CountryMap.vue'
import HeroTotal from '@/components/HeroTotal.vue'
import StatCard from '@/components/StatCard.vue'
import StatCardSkeleton from '@/components/StatCardSkeleton.vue'
import { useStats } from '@/composables/useStats'
import { formatColumnLabel } from '@/utils/format'

const { data, loading, error } = useStats()
</script>

<template>
  <div class="mx-auto flex w-full container max-w-6xl flex-col gap-6">
    <HeroTotal :value="data?.total ?? 0" />

    <p
      v-if="error"
      class="rounded-2xl bg-surface-elevated p-6 text-center text-sm text-on-surface-muted"
    >
      Couldn&rsquo;t load statistics. {{ error.message }}
    </p>

    <div v-else-if="loading && !data" class="grid gap-4 md:grid-cols-2 xl:grid-cols-4">
      <StatCardSkeleton
        v-for="column in FILTER_COLUMNS"
        :key="column"
        :title="`Top ${formatColumnLabel(column)}`"
      />
    </div>

    <template v-else-if="data">
      <div class="grid gap-4 md:grid-cols-2 xl:grid-cols-4">
        <StatCard
          v-for="column in FILTER_COLUMNS"
          v-show="Object.keys(data[column]).length"
          :key="column"
          :title="`Top ${formatColumnLabel(column)}`"
          :column="column"
          :entries="data[column]"
        />
      </div>
      <CountryMap :entries="data.country" :total="data.total" />
    </template>
  </div>
</template>

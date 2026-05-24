<!--
SPDX-FileCopyrightText: 2026 The LineageOS Project

SPDX-License-Identifier: Apache-2.0
-->

<script setup lang="ts">
import { FILTER_COLUMNS, FILTER_TITLES } from '@/api/types'
import CountryMap from '@/components/CountryMap.vue'
import HeroTotal from '@/components/HeroTotal.vue'
import StatCard from '@/components/StatCard.vue'
import StatCardSkeleton from '@/components/StatCardSkeleton.vue'
import { useStats } from '@/composables/useStats'

const { data, loading, error } = useStats()
</script>

<template>
  <div class="mx-auto flex max-w-7xl flex-col gap-6">
    <HeroTotal label="Total active installs" :value="data?.total ?? 0" />

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
        :title="FILTER_TITLES[column]"
      />
    </div>

    <template v-else-if="data">
      <div class="grid gap-4 md:grid-cols-2 xl:grid-cols-4">
        <StatCard :title="FILTER_TITLES.model" column="model" :entries="data.model" />
        <StatCard :title="FILTER_TITLES.country" column="country" :entries="data.country" />
        <StatCard :title="FILTER_TITLES.version" column="version" :entries="data.version" />
        <StatCard
          v-if="data.carrier && Object.keys(data.carrier).length"
          :title="FILTER_TITLES.carrier"
          column="carrier"
          :entries="data.carrier"
        />
      </div>
      <CountryMap :entries="data.country" :total="data.total" />
    </template>
  </div>
</template>

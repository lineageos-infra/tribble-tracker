<!--
SPDX-FileCopyrightText: 2026 The LineageOS Project

SPDX-License-Identifier: Apache-2.0
-->

<script setup lang="ts">
import CountryMap from '@/components/CountryMap.vue'
import HeroTotal from '@/components/HeroTotal.vue'
import StatCard from '@/components/StatCard.vue'
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

    <p
      v-else-if="loading && !data"
      class="rounded-2xl bg-surface-elevated p-6 text-center text-sm text-on-surface-muted"
    >
      Loading…
    </p>

    <template v-else-if="data">
      <div class="grid gap-4 md:grid-cols-2 xl:grid-cols-4">
        <StatCard title="Top Devices" column="model" :entries="data.model" />
        <StatCard title="Top Countries" column="country" :entries="data.country" />
        <StatCard title="Top Versions" column="version" :entries="data.version" />
        <StatCard
          v-if="data.carrier && Object.keys(data.carrier).length"
          title="Top Carriers"
          column="carrier"
          :entries="data.carrier"
        />
      </div>
      <CountryMap :entries="data.country" :total="data.total" />
    </template>
  </div>
</template>

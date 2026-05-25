<!--
SPDX-FileCopyrightText: 2026 The LineageOS Project

SPDX-License-Identifier: Apache-2.0
-->

<script setup lang="ts">
defineProps<{
  title: string
}>()
</script>

<template>
  <section class="flex h-full flex-col rounded-3xl bg-surface-elevated p-4 sm:p-5" aria-busy="true">
    <header class="mb-3 flex items-baseline justify-between gap-2 px-2">
      <h2 class="text-lg font-medium text-on-surface">{{ title }}</h2>
      <span class="h-3 w-8 animate-pulse rounded-full bg-outline-variant" />
    </header>

    <div class="mb-3 px-1">
      <div class="h-9 w-full animate-pulse rounded-full bg-outline-variant" />
    </div>

    <div class="-mx-1 flex-1 space-y-2 overflow-hidden pr-1">
      <div v-for="i in 8" :key="i" class="flex items-center gap-3 px-2 py-1.5">
        <span class="h-3 w-4 animate-pulse rounded bg-outline-variant" />
        <span
          class="h-3 animate-pulse rounded bg-outline-variant"
          :style="{ width: `${90 - i * 8}%` }"
        />
      </div>
    </div>
  </section>
</template>

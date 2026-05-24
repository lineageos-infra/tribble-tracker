<!--
SPDX-FileCopyrightText: 2026 The LineageOS Project

SPDX-License-Identifier: Apache-2.0
-->

<script setup lang="ts">
import { ChevronLeft } from '@lucide/vue'
</script>

<template>
  <div
    class="mx-auto flex w-full max-w-7xl flex-1 flex-col items-center justify-center gap-6 text-center"
  >
    <h1 class="text-4xl">Page Not Found</h1>
    <h2>
      Sorry, but the page you were trying to view does not exist. Try searching for it or looking at
      the URL to see if it looks correct.
    </h2>
    <RouterLink
      to="/"
      class="inline-flex h-12 items-center gap-3 rounded-lg bg-brand-light pr-6 pl-3 text-base leading-5 font-medium text-brand-dark transition focus:outline-none focus-visible:ring-2 focus-visible:ring-brand-primary hover:brightness-95 active:brightness-90 dark:bg-brand-dark dark:text-brand-light dark:hover:brightness-110 dark:active:brightness-125"
    >
      <ChevronLeft class="size-6" :stroke-width="2" aria-hidden="true" />
      <span>Back to home</span>
    </RouterLink>
  </div>
</template>

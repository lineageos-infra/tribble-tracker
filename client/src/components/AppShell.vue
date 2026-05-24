<!--
SPDX-FileCopyrightText: 2026 The LineageOS Project

SPDX-License-Identifier: Apache-2.0
-->

<script setup lang="ts">
import AppFooter from './AppFooter.vue'
import TopAppBar from './TopAppBar.vue'
</script>

<template>
  <div class="flex min-h-dvh flex-col bg-surface text-on-surface">
    <TopAppBar />
    <main class="flex flex-1 flex-col px-4 py-6 sm:px-8 sm:py-10">
      <slot />
    </main>
    <AppFooter />
  </div>
</template>

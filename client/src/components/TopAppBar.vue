<!--
SPDX-FileCopyrightText: 2026 The LineageOS Project

SPDX-License-Identifier: Apache-2.0
-->

<script setup lang="ts">
import navbarLogo from '@/assets/navbar-logo.svg'
import { ExternalLink } from '@lucide/vue'
</script>

<template>
  <header class="sticky top-0 z-20 border-b border-outline-variant bg-surface px-4 sm:px-8">
    <div class="mx-auto flex h-16 max-w-7xl items-center gap-3">
      <RouterLink
        to="/"
        class="flex items-center gap-3 rounded-full px-2 py-1 focus:outline-none focus-visible:ring-2 focus-visible:ring-brand-primary"
        aria-label="LineageOS Statistics — home"
      >
        <img :src="navbarLogo" alt="" class="h-6 w-auto" aria-hidden="true" />
        <span class="text-base font-medium text-on-surface">LineageOS Statistics</span>
      </RouterLink>

      <div class="ml-auto flex items-center gap-1">
        <a
          href="https://lineageos.org"
          target="_blank"
          rel="noopener"
          class="hidden items-center gap-1.5 rounded-full px-4 py-2 text-xs font-medium text-on-surface-muted uppercase focus:outline-none focus-visible:ring-2 focus-visible:ring-brand-primary md:inline-flex"
        >
          lineageos.org
          <ExternalLink class="size-4" aria-hidden="true" />
        </a>
      </div>
    </div>
  </header>
</template>

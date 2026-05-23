<script setup lang="ts">
import navbarLogo from '@/assets/navbar-logo.svg'

const year = new Date().getFullYear()
</script>

<template>
  <footer
    class="mt-12 border-t border-outline-variant px-4 py-8 text-sm text-on-surface-muted sm:px-8"
  >
    <div
      class="mx-auto flex max-w-7xl flex-col items-center gap-4 text-center md:flex-row md:items-center md:justify-between md:text-left"
    >
      <div class="flex flex-col items-center gap-3 md:items-start">
        <img :src="navbarLogo" alt="LineageOS" class="h-5 w-auto opacity-70" />
        <p>&copy; 2016 &ndash; {{ year }} The LineageOS Project</p>
      </div>
      <div class="flex flex-col gap-1 md:text-right">
        <p>
          Licensed under
          <a
            href="https://creativecommons.org/licenses/by-sa/3.0/"
            target="_blank"
            rel="noopener"
            class="text-brand-primary hover:underline"
            >CC BY-SA 3.0</a
          >.
        </p>
        <p>Data refreshed hourly</p>
      </div>
    </div>
  </footer>
</template>

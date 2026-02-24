<script lang="ts">
  let { size = 32 }: { size?: number } = $props();
  const uid = Math.random().toString(36).slice(2, 8);
  const gid = `pg-${uid}`;
</script>

<svg
  width={size}
  height={size}
  viewBox="0 0 32 32"
  fill="none"
  xmlns="http://www.w3.org/2000/svg"
  class="logo-mark"
  aria-hidden="true"
>
  <defs>
    <linearGradient id={gid} x1="0" y1="0" x2="32" y2="32" gradientUnits="userSpaceOnUse">
      <stop stop-color="#E8924A"/>
      <stop offset="1" stop-color="#E85D5D"/>
    </linearGradient>
  </defs>
  <!-- Outer ring: 3 arcs with gaps forming an aperture -->
  <path d="M28.8 13.7 A13 13 0 0 1 13.7 28.8" stroke="url(#{gid})" stroke-width="3" stroke-linecap="round"/>
  <path d="M11.6 28.2 A13 13 0 0 1 6 7.6" stroke="url(#{gid})" stroke-width="3" stroke-linecap="round"/>
  <path d="M7.6 6 A13 13 0 0 1 28.2 11.6" stroke="url(#{gid})" stroke-width="3" stroke-linecap="round"/>
  <!-- Inner ring: 3 arcs offset 60° for depth -->
  <path d="M20.5 21.4 A7 7 0 0 1 9.4 18.4" stroke="url(#{gid})" stroke-width="2" stroke-linecap="round" opacity="0.5"/>
  <path d="M9.1 17.2 A7 7 0 0 1 17.2 9.1" stroke="url(#{gid})" stroke-width="2" stroke-linecap="round" opacity="0.5"/>
  <path d="M18.4 9.4 A7 7 0 0 1 21.4 20.5" stroke="url(#{gid})" stroke-width="2" stroke-linecap="round" opacity="0.5"/>
</svg>

<style>
  .logo-mark {
    display: block;
    flex-shrink: 0;
  }
</style>

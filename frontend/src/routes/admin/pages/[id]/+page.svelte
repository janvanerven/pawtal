<script lang="ts">
  import type { PageData } from './$types';
  import PageEditor from '$lib/components/PageEditor.svelte';

  let { data }: { data: PageData } = $props();
</script>

<svelte:head>
  <title>{data.page.title} — Pawtal CMS</title>
</svelte:head>

<PageEditor page={data.page} categories={data.categories} />

<script lang="ts">
  import type { PageData } from './$types';
  import PageEditor from '$lib/components/PageEditor.svelte';

  let { data }: { data: PageData } = $props();
</script>

<svelte:head>
  <title>New Page — Pawtal CMS</title>
</svelte:head>

<PageEditor categories={data.categories} />

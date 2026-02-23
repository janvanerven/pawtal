<script lang="ts">
  import type { PageData } from './$types';
  import ArticleEditor from '$lib/components/ArticleEditor.svelte';

  let { data }: { data: PageData } = $props();
</script>

<svelte:head>
  <title>New Article — Pawtal CMS</title>
</svelte:head>

<ArticleEditor categories={data.categories} />

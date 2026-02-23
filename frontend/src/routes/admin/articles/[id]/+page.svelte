<script lang="ts">
  import type { PageData } from './$types';
  import ArticleEditor from '$lib/components/ArticleEditor.svelte';

  let { data }: { data: PageData } = $props();
</script>

<svelte:head>
  <title>{data.article.title} — Pawtal CMS</title>
</svelte:head>

<ArticleEditor article={data.article} categories={data.categories} />

<script lang="ts">
  import { onMount, onDestroy, createEventDispatcher } from 'svelte';
  import { Editor } from '@tiptap/core';
  import StarterKit from '@tiptap/starter-kit';
  import Link from '@tiptap/extension-link';
  import Image from '@tiptap/extension-image';
  import Placeholder from '@tiptap/extension-placeholder';

  interface Props {
    content: string;
    onUpdate?: (html: string) => void;
    placeholder?: string;
  }

  let { content, onUpdate, placeholder = 'Start writing...' }: Props = $props();

  const dispatch = createEventDispatcher<{ 'insert-image': void }>();

  let editorElement: HTMLElement;
  let editor: Editor | undefined;

  // Track active states for toolbar
  let isBold = $state(false);
  let isItalic = $state(false);
  let isH1 = $state(false);
  let isH2 = $state(false);
  let isH3 = $state(false);
  let isBullet = $state(false);
  let isOrdered = $state(false);
  let isBlockquote = $state(false);
  let isCodeBlock = $state(false);

  function updateActiveStates() {
    if (!editor) return;
    isBold = editor.isActive('bold');
    isItalic = editor.isActive('italic');
    isH1 = editor.isActive('heading', { level: 1 });
    isH2 = editor.isActive('heading', { level: 2 });
    isH3 = editor.isActive('heading', { level: 3 });
    isBullet = editor.isActive('bulletList');
    isOrdered = editor.isActive('orderedList');
    isBlockquote = editor.isActive('blockquote');
    isCodeBlock = editor.isActive('codeBlock');
  }

  onMount(() => {
    editor = new Editor({
      element: editorElement,
      extensions: [
        StarterKit,
        Link.configure({ openOnClick: false }),
        Image,
        Placeholder.configure({ placeholder }),
      ],
      content,
      onUpdate: ({ editor: e }) => {
        onUpdate?.(e.getHTML());
        updateActiveStates();
      },
      onSelectionUpdate: () => updateActiveStates(),
      onTransaction: () => updateActiveStates(),
    });
  });

  onDestroy(() => {
    editor?.destroy();
  });

  // Sync external content changes (e.g., when loading a saved revision)
  $effect(() => {
    if (editor && content !== editor.getHTML()) {
      editor.commands.setContent(content);
    }
  });

  function cmd(action: () => void) {
    action();
    editor?.commands.focus();
  }

  function insertLink() {
    const url = window.prompt('Enter URL:');
    if (!url) return;
    if (editor?.state.selection.empty) {
      editor.chain().focus().insertContent(`<a href="${url}">${url}</a>`).run();
    } else {
      editor?.chain().focus().setLink({ href: url }).run();
    }
  }
</script>

<div class="rte-wrapper">
  <!-- Toolbar -->
  <div class="toolbar">
    <div class="toolbar-group">
      <button
        type="button"
        class="toolbar-btn"
        class:active={isBold}
        title="Bold"
        onclick={() => cmd(() => editor?.chain().focus().toggleBold().run())}
      ><strong>B</strong></button>
      <button
        type="button"
        class="toolbar-btn"
        class:active={isItalic}
        title="Italic"
        onclick={() => cmd(() => editor?.chain().focus().toggleItalic().run())}
      ><em>I</em></button>
    </div>

    <div class="toolbar-separator"></div>

    <div class="toolbar-group">
      <button
        type="button"
        class="toolbar-btn"
        class:active={isH1}
        title="Heading 1"
        onclick={() => cmd(() => editor?.chain().focus().toggleHeading({ level: 1 }).run())}
      >H1</button>
      <button
        type="button"
        class="toolbar-btn"
        class:active={isH2}
        title="Heading 2"
        onclick={() => cmd(() => editor?.chain().focus().toggleHeading({ level: 2 }).run())}
      >H2</button>
      <button
        type="button"
        class="toolbar-btn"
        class:active={isH3}
        title="Heading 3"
        onclick={() => cmd(() => editor?.chain().focus().toggleHeading({ level: 3 }).run())}
      >H3</button>
    </div>

    <div class="toolbar-separator"></div>

    <div class="toolbar-group">
      <button
        type="button"
        class="toolbar-btn"
        class:active={isBullet}
        title="Bullet List"
        onclick={() => cmd(() => editor?.chain().focus().toggleBulletList().run())}
      >≡</button>
      <button
        type="button"
        class="toolbar-btn"
        class:active={isOrdered}
        title="Ordered List"
        onclick={() => cmd(() => editor?.chain().focus().toggleOrderedList().run())}
      >1.</button>
      <button
        type="button"
        class="toolbar-btn"
        class:active={isBlockquote}
        title="Blockquote"
        onclick={() => cmd(() => editor?.chain().focus().toggleBlockquote().run())}
      >"</button>
      <button
        type="button"
        class="toolbar-btn"
        class:active={isCodeBlock}
        title="Code Block"
        onclick={() => cmd(() => editor?.chain().focus().toggleCodeBlock().run())}
      >&lt;/&gt;</button>
    </div>

    <div class="toolbar-separator"></div>

    <div class="toolbar-group">
      <button
        type="button"
        class="toolbar-btn"
        title="Insert Link"
        onclick={insertLink}
      >🔗</button>
      <button
        type="button"
        class="toolbar-btn"
        title="Insert Image"
        onclick={() => dispatch('insert-image')}
      >🖼</button>
      <button
        type="button"
        class="toolbar-btn"
        title="Horizontal Rule"
        onclick={() => cmd(() => editor?.chain().focus().setHorizontalRule().run())}
      >—</button>
    </div>
  </div>

  <!-- Editor content area -->
  <div class="editor-area" bind:this={editorElement}></div>
</div>

<style>
  .rte-wrapper {
    border: 1px solid var(--color-border);
    border-radius: var(--radius-md);
    overflow: hidden;
    background: var(--color-surface);
  }

  .rte-wrapper:focus-within {
    border-color: var(--color-primary);
    box-shadow: 0 0 0 3px rgba(232, 146, 74, 0.15);
  }

  .toolbar {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 2px;
    padding: var(--space-sm);
    border-bottom: 1px solid var(--color-border);
    background: var(--color-bg);
  }

  .toolbar-group {
    display: flex;
    gap: 2px;
  }

  .toolbar-separator {
    width: 1px;
    height: 24px;
    background: var(--color-border);
    margin: 0 var(--space-xs);
  }

  .toolbar-btn {
    padding: 4px 8px;
    border-radius: var(--radius-sm);
    font-size: 0.8rem;
    font-family: var(--font-body);
    color: var(--color-text-muted);
    background: transparent;
    border: 1px solid transparent;
    cursor: pointer;
    transition: all var(--transition-fast);
    min-width: 28px;
    text-align: center;
    line-height: 1.5;
  }

  .toolbar-btn:hover {
    background: var(--color-surface);
    color: var(--color-text);
    border-color: var(--color-border);
  }

  .toolbar-btn.active {
    background: rgba(232, 146, 74, 0.15);
    color: var(--color-primary);
    border-color: rgba(232, 146, 74, 0.3);
  }

  .editor-area {
    min-height: 320px;
    padding: var(--space-lg);
    font-family: var(--font-body);
    font-size: 1rem;
    line-height: 1.7;
    color: var(--color-text);
    cursor: text;
  }

  /* TipTap editor content styles */
  .editor-area :global(.tiptap) {
    outline: none;
    min-height: 280px;
  }

  .editor-area :global(.tiptap p) {
    margin-bottom: var(--space-md);
  }

  .editor-area :global(.tiptap h1) {
    font-size: 1.75rem;
    margin-bottom: var(--space-md);
    margin-top: var(--space-lg);
  }

  .editor-area :global(.tiptap h2) {
    font-size: 1.375rem;
    margin-bottom: var(--space-sm);
    margin-top: var(--space-lg);
  }

  .editor-area :global(.tiptap h3) {
    font-size: 1.125rem;
    margin-bottom: var(--space-sm);
    margin-top: var(--space-md);
  }

  .editor-area :global(.tiptap ul),
  .editor-area :global(.tiptap ol) {
    padding-left: var(--space-xl);
    margin-bottom: var(--space-md);
  }

  .editor-area :global(.tiptap li) {
    margin-bottom: var(--space-xs);
  }

  .editor-area :global(.tiptap blockquote) {
    border-left: 4px solid var(--color-primary);
    padding-left: var(--space-md);
    margin: var(--space-md) 0;
    color: var(--color-text-muted);
    font-style: italic;
  }

  .editor-area :global(.tiptap pre) {
    background: var(--color-bg);
    border-radius: var(--radius-sm);
    padding: var(--space-md);
    font-family: monospace;
    font-size: 0.875rem;
    overflow-x: auto;
    margin-bottom: var(--space-md);
  }

  .editor-area :global(.tiptap code) {
    background: var(--color-bg);
    padding: 1px 6px;
    border-radius: 4px;
    font-family: monospace;
    font-size: 0.875em;
  }

  .editor-area :global(.tiptap hr) {
    border: none;
    border-top: 2px solid var(--color-border);
    margin: var(--space-xl) 0;
  }

  .editor-area :global(.tiptap img) {
    max-width: 100%;
    border-radius: var(--radius-sm);
  }

  .editor-area :global(.tiptap a) {
    color: var(--color-primary);
    text-decoration: underline;
  }

  /* Placeholder */
  .editor-area :global(.tiptap p.is-editor-empty:first-child::before) {
    content: attr(data-placeholder);
    color: var(--color-text-light);
    pointer-events: none;
    float: left;
    height: 0;
  }
</style>

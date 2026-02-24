<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { Editor } from '@tiptap/core';
  import StarterKit from '@tiptap/starter-kit';
  import Link from '@tiptap/extension-link';
  import Image from '@tiptap/extension-image';
  import Placeholder from '@tiptap/extension-placeholder';
  import CharacterCount from '@tiptap/extension-character-count';
  import BubbleMenu from '@tiptap/extension-bubble-menu';
  import Dropcursor from '@tiptap/extension-dropcursor';

  interface Props {
    content: string;
    onUpdate?: (html: string) => void;
    onInsertImage?: () => void;
    placeholder?: string;
  }

  let { content, onUpdate, onInsertImage, placeholder = 'Start writing... (type / for commands)' }: Props = $props();

  let editorElement: HTMLElement;
  let bubbleMenuElement: HTMLElement;
  let editor = $state<Editor | undefined>(undefined);

  // Slash command state
  let slashOpen = $state(false);
  let slashQuery = $state('');
  let slashIndex = $state(0);
  let slashPos = $state({ top: 0, left: 0 });

  const slashCommands = [
    { label: 'Heading 1', icon: 'H1', action: (e: Editor) => e.chain().focus().toggleHeading({ level: 1 }).run() },
    { label: 'Heading 2', icon: 'H2', action: (e: Editor) => e.chain().focus().toggleHeading({ level: 2 }).run() },
    { label: 'Heading 3', icon: 'H3', action: (e: Editor) => e.chain().focus().toggleHeading({ level: 3 }).run() },
    { label: 'Bullet List', icon: '•', action: (e: Editor) => e.chain().focus().toggleBulletList().run() },
    { label: 'Ordered List', icon: '1.', action: (e: Editor) => e.chain().focus().toggleOrderedList().run() },
    { label: 'Blockquote', icon: '"', action: (e: Editor) => e.chain().focus().toggleBlockquote().run() },
    { label: 'Code Block', icon: '</>', action: (e: Editor) => e.chain().focus().toggleCodeBlock().run() },
    { label: 'Horizontal Rule', icon: '—', action: (e: Editor) => e.chain().focus().setHorizontalRule().run() },
    { label: 'Image', icon: '🖼', action: () => onInsertImage?.() },
  ];

  let filteredCommands = $derived(
    slashQuery
      ? slashCommands.filter(c => c.label.toLowerCase().includes(slashQuery.toLowerCase()))
      : slashCommands
  );

  function handleInsertImage(e: Event) {
    const { src, alt } = (e as CustomEvent).detail;
    editor?.chain().focus().setImage({ src, alt }).run();
  }

  function executeSlashCommand(index: number) {
    if (!editor || !filteredCommands[index]) return;
    // Delete the slash and query text
    const { from } = editor.state.selection;
    const deleteFrom = from - slashQuery.length - 1; // -1 for the '/'
    editor.chain().focus().deleteRange({ from: deleteFrom, to: from }).run();
    filteredCommands[index].action(editor);
    slashOpen = false;
    slashQuery = '';
    slashIndex = 0;
  }

  function getCaretCoords() {
    const sel = window.getSelection();
    if (!sel || !sel.rangeCount) return { top: 0, left: 0 };
    const range = sel.getRangeAt(0);
    const rect = range.getBoundingClientRect();
    const editorRect = editorElement.getBoundingClientRect();
    return {
      top: rect.bottom - editorRect.top + 8,
      left: rect.left - editorRect.left,
    };
  }

  function handleDrop(view: any, event: DragEvent) {
    const files = event.dataTransfer?.files;
    if (!files?.length) return false;

    const imageFiles = Array.from(files).filter(f => f.type.startsWith('image/'));
    if (!imageFiles.length) return false;

    event.preventDefault();

    // Upload asynchronously, but return synchronously
    for (const file of imageFiles) {
      const formData = new FormData();
      formData.append('file', file);
      fetch('/api/admin/media', {
        method: 'POST',
        body: formData,
      }).then(resp => {
        if (resp.ok) return resp.json();
      }).then(media => {
        if (media) {
          const src = `/uploads/${media.id}/${media.filename}`;
          editor?.chain().focus().setImage({ src, alt: media.original_filename || file.name }).run();
        }
      }).catch(() => {
        // silent fail for drag-drop
      });
    }
    return true;
  }

  onMount(() => {
    editor = new Editor({
      element: editorElement,
      extensions: [
        StarterKit,
        Link.configure({ openOnClick: false }),
        Image,
        Placeholder.configure({ placeholder }),
        CharacterCount,
        BubbleMenu.configure({ element: bubbleMenuElement }),
        Dropcursor.configure({ color: 'var(--color-primary)', width: 2 }),
      ],
      content,
      editorProps: {
        handleDrop,
        handleKeyDown: (_view: any, event: KeyboardEvent) => {
          if (slashOpen) {
            if (event.key === 'ArrowDown') {
              event.preventDefault();
              slashIndex = Math.min(slashIndex + 1, filteredCommands.length - 1);
              return true;
            }
            if (event.key === 'ArrowUp') {
              event.preventDefault();
              slashIndex = Math.max(slashIndex - 1, 0);
              return true;
            }
            if (event.key === 'Enter') {
              event.preventDefault();
              executeSlashCommand(slashIndex);
              return true;
            }
            if (event.key === 'Escape') {
              event.preventDefault();
              slashOpen = false;
              slashQuery = '';
              return true;
            }
          }
          return false;
        },
      },
      onUpdate: ({ editor: e }) => {
        onUpdate?.(e.getHTML());
        checkSlashCommand(e);
      },
      onSelectionUpdate: ({ editor: e }) => {
        checkSlashCommand(e);
      },
    });

    window.addEventListener('rte:insert-image', handleInsertImage);
  });

  function checkSlashCommand(e: Editor) {
    const { from } = e.state.selection;
    const textBefore = e.state.doc.textBetween(
      Math.max(0, from - 20),
      from,
      '\n'
    );
    const slashMatch = textBefore.match(/\/([a-zA-Z0-9]*)$/);
    if (slashMatch) {
      slashQuery = slashMatch[1];
      slashIndex = 0;
      slashPos = getCaretCoords();
      slashOpen = true;
    } else {
      slashOpen = false;
      slashQuery = '';
    }
  }

  onDestroy(() => {
    window.removeEventListener('rte:insert-image', handleInsertImage);
    editor?.destroy();
  });

  $effect(() => {
    if (editor && content !== editor.getHTML()) {
      editor.commands.setContent(content, { emitUpdate: false });
    }
  });

  function insertLink() {
    const url = window.prompt('Enter URL:');
    if (!url) return;
    // Validate URL scheme to prevent javascript: XSS
    try {
      const parsed = new URL(url, window.location.origin);
      if (!['http:', 'https:', 'mailto:'].includes(parsed.protocol)) return;
    } catch {
      return;
    }
    if (editor?.state.selection.empty) {
      // Use TipTap API instead of raw HTML interpolation to prevent attribute injection
      editor.chain().focus().insertContent(url).setLink({ href: url }).run();
    } else {
      editor?.chain().focus().setLink({ href: url }).run();
    }
  }

  let wordCount = $derived(editor?.storage.characterCount?.words() ?? 0);
</script>

<div class="rte-wrapper">
  <!-- Bubble Menu (floating toolbar on selection) -->
  <div class="bubble-menu" bind:this={bubbleMenuElement}>
    <button
      type="button"
      class="bubble-btn"
      class:active={editor?.isActive('bold')}
      onclick={() => editor?.chain().focus().toggleBold().run()}
    ><strong>B</strong></button>
    <button
      type="button"
      class="bubble-btn"
      class:active={editor?.isActive('italic')}
      onclick={() => editor?.chain().focus().toggleItalic().run()}
    ><em>I</em></button>
    <span class="bubble-sep"></span>
    <button
      type="button"
      class="bubble-btn"
      class:active={editor?.isActive('heading', { level: 2 })}
      onclick={() => editor?.chain().focus().toggleHeading({ level: 2 }).run()}
    >H2</button>
    <button
      type="button"
      class="bubble-btn"
      class:active={editor?.isActive('heading', { level: 3 })}
      onclick={() => editor?.chain().focus().toggleHeading({ level: 3 }).run()}
    >H3</button>
    <span class="bubble-sep"></span>
    <button
      type="button"
      class="bubble-btn"
      onclick={insertLink}
    >Link</button>
    <button
      type="button"
      class="bubble-btn"
      class:active={editor?.isActive('code')}
      onclick={() => editor?.chain().focus().toggleCode().run()}
    >Code</button>
    <button
      type="button"
      class="bubble-btn"
      class:active={editor?.isActive('blockquote')}
      onclick={() => editor?.chain().focus().toggleBlockquote().run()}
    >"</button>
  </div>

  <!-- Minimal top bar with undo/redo and image insert -->
  <div class="toolbar-top">
    <div class="toolbar-left">
      <button
        type="button"
        class="toolbar-btn"
        title="Undo (Ctrl+Z)"
        onclick={() => editor?.chain().focus().undo().run()}
        disabled={!editor?.can().undo()}
      >
        <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="1 4 1 10 7 10"/><path d="M3.51 15a9 9 0 1 0 2.13-9.36L1 10"/></svg>
      </button>
      <button
        type="button"
        class="toolbar-btn"
        title="Redo (Ctrl+Shift+Z)"
        onclick={() => editor?.chain().focus().redo().run()}
        disabled={!editor?.can().redo()}
      >
        <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="23 4 23 10 17 10"/><path d="M20.49 15a9 9 0 1 1-2.12-9.36L23 10"/></svg>
      </button>
      <span class="toolbar-sep"></span>
      <button
        type="button"
        class="toolbar-btn"
        title="Insert Image"
        onclick={() => onInsertImage?.()}
      >
        <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="3" y="3" width="18" height="18" rx="2" ry="2"/><circle cx="8.5" cy="8.5" r="1.5"/><polyline points="21 15 16 10 5 21"/></svg>
      </button>
      <button
        type="button"
        class="toolbar-btn"
        title="Insert Link"
        onclick={insertLink}
      >
        <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M10 13a5 5 0 0 0 7.54.54l3-3a5 5 0 0 0-7.07-7.07l-1.72 1.71"/><path d="M14 11a5 5 0 0 0-7.54-.54l-3 3a5 5 0 0 0 7.07 7.07l1.71-1.71"/></svg>
      </button>
    </div>
    <div class="toolbar-hint">
      Type <kbd>/</kbd> for commands
    </div>
  </div>

  <!-- Editor content area -->
  <div class="editor-area" bind:this={editorElement}></div>

  <!-- Slash command dropdown -->
  {#if slashOpen && filteredCommands.length > 0}
    <div class="slash-menu" style="top: {slashPos.top}px; left: {slashPos.left}px">
      {#each filteredCommands as cmd, i (cmd.label)}
        <button
          type="button"
          class="slash-item"
          class:selected={i === slashIndex}
          onmouseenter={() => slashIndex = i}
          onclick={() => executeSlashCommand(i)}
        >
          <span class="slash-icon">{cmd.icon}</span>
          <span class="slash-label">{cmd.label}</span>
        </button>
      {/each}
    </div>
  {/if}

  <!-- Footer with word count -->
  <div class="editor-footer">
    <span class="word-count">{wordCount} words</span>
  </div>
</div>

<style>
  .rte-wrapper {
    border: 1px solid var(--color-border);
    border-radius: var(--radius-md);
    overflow: visible;
    background: var(--color-surface);
    position: relative;
  }

  .rte-wrapper:focus-within {
    border-color: var(--color-primary);
    box-shadow: 0 0 0 3px rgba(232, 146, 74, 0.15);
  }

  /* Bubble menu (floating toolbar) */
  .bubble-menu {
    display: flex;
    align-items: center;
    gap: 2px;
    padding: 4px 6px;
    background: var(--color-surface-elevated);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-md);
    box-shadow: var(--shadow-md);
  }

  .bubble-btn {
    padding: 4px 8px;
    border-radius: var(--radius-sm);
    font-size: 0.78rem;
    font-family: var(--font-body);
    color: var(--color-text-muted);
    background: transparent;
    border: none;
    cursor: pointer;
    min-width: 28px;
    text-align: center;
    transition: all var(--transition-fast);
  }

  .bubble-btn:hover {
    background: var(--color-bg);
    color: var(--color-text);
  }

  .bubble-btn.active {
    color: var(--color-primary);
    background: rgba(232, 146, 74, 0.12);
  }

  .bubble-sep {
    width: 1px;
    height: 18px;
    background: var(--color-border);
    margin: 0 2px;
  }

  /* Top toolbar (minimal) */
  .toolbar-top {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: var(--space-xs) var(--space-sm);
    border-bottom: 1px solid var(--color-border);
    background: var(--color-bg);
  }

  .toolbar-left {
    display: flex;
    align-items: center;
    gap: 2px;
  }

  .toolbar-btn {
    padding: 6px;
    border-radius: var(--radius-sm);
    color: var(--color-text-muted);
    background: transparent;
    border: none;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: all var(--transition-fast);
  }

  .toolbar-btn:hover:not(:disabled) {
    background: var(--color-surface);
    color: var(--color-text);
  }

  .toolbar-btn:disabled {
    opacity: 0.3;
    cursor: default;
  }

  .toolbar-sep {
    width: 1px;
    height: 20px;
    background: var(--color-border);
    margin: 0 4px;
  }

  .toolbar-hint {
    font-size: 0.72rem;
    color: var(--color-text-light);
  }

  .toolbar-hint kbd {
    font-family: var(--font-body);
    font-size: 0.7rem;
    padding: 1px 5px;
    border-radius: 3px;
    background: var(--color-surface);
    border: 1px solid var(--color-border);
  }

  /* Editor area */
  .editor-area {
    min-height: 320px;
    padding: var(--space-lg);
    font-family: var(--font-body);
    font-size: 1rem;
    line-height: 1.7;
    color: var(--color-text);
    cursor: text;
  }

  .editor-area :global(.tiptap) { outline: none; min-height: 280px; }
  .editor-area :global(.tiptap p) { margin-bottom: var(--space-md); }
  .editor-area :global(.tiptap h1) { font-size: 1.75rem; margin-bottom: var(--space-md); margin-top: var(--space-lg); }
  .editor-area :global(.tiptap h2) { font-size: 1.375rem; margin-bottom: var(--space-sm); margin-top: var(--space-lg); }
  .editor-area :global(.tiptap h3) { font-size: 1.125rem; margin-bottom: var(--space-sm); margin-top: var(--space-md); }
  .editor-area :global(.tiptap ul),
  .editor-area :global(.tiptap ol) { padding-left: var(--space-xl); margin-bottom: var(--space-md); }
  .editor-area :global(.tiptap li) { margin-bottom: var(--space-xs); }
  .editor-area :global(.tiptap blockquote) { border-left: 4px solid var(--color-primary); padding-left: var(--space-md); margin: var(--space-md) 0; color: var(--color-text-muted); font-style: italic; }
  .editor-area :global(.tiptap pre) { background: var(--color-bg); border-radius: var(--radius-sm); padding: var(--space-md); font-family: monospace; font-size: 0.875rem; overflow-x: auto; margin-bottom: var(--space-md); }
  .editor-area :global(.tiptap code) { background: var(--color-bg); padding: 1px 6px; border-radius: 4px; font-family: monospace; font-size: 0.875em; }
  .editor-area :global(.tiptap hr) { border: none; border-top: 2px solid var(--color-border); margin: var(--space-xl) 0; }
  .editor-area :global(.tiptap img) { max-width: 100%; border-radius: var(--radius-sm); }
  .editor-area :global(.tiptap a) { color: var(--color-primary); text-decoration: underline; }
  .editor-area :global(.tiptap p.is-editor-empty:first-child::before) { content: attr(data-placeholder); color: var(--color-text-light); pointer-events: none; float: left; height: 0; }

  /* Slash command menu */
  .slash-menu {
    position: absolute;
    z-index: 50;
    background: var(--color-surface);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-md);
    box-shadow: var(--shadow-lg);
    padding: var(--space-xs);
    min-width: 200px;
    max-height: 280px;
    overflow-y: auto;
  }

  .slash-item {
    display: flex;
    align-items: center;
    gap: var(--space-sm);
    width: 100%;
    padding: var(--space-sm) var(--space-md);
    border: none;
    background: transparent;
    border-radius: var(--radius-sm);
    cursor: pointer;
    color: var(--color-text);
    font-size: 0.85rem;
    text-align: left;
    transition: background var(--transition-fast);
  }

  .slash-item:hover,
  .slash-item.selected {
    background: var(--color-bg);
  }

  .slash-icon {
    width: 28px;
    height: 28px;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: var(--radius-sm);
    background: var(--color-bg);
    border: 1px solid var(--color-border);
    font-size: 0.75rem;
    font-weight: 600;
    color: var(--color-text-muted);
    flex-shrink: 0;
  }

  /* Editor footer */
  .editor-footer {
    padding: var(--space-xs) var(--space-md);
    border-top: 1px solid var(--color-border);
    background: var(--color-bg);
    display: flex;
    justify-content: flex-end;
  }

  .word-count {
    font-size: 0.72rem;
    color: var(--color-text-light);
  }
</style>

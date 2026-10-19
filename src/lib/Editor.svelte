<script lang="ts">
  import { save } from "@tauri-apps/plugin-dialog";
  import Keycap from "./Keycap.svelte";
  import { api, emptyPreset, type Binding, type Preset } from "./api";
  import { conflicts, fromKey, fromMouse, fromWheel, group, GROUP_ORDER, label } from "./keys";

  let { account, version, onreview, notify }: {
    account: number; version: number; onreview: (p: Preset) => void; notify: (msg: string, error?: boolean) => void;
  } = $props();

  let base = $state<Binding[]>([]);
  let draft = $state<Record<string, string | null>>({});
  let capturing = $state<string | null>(null);
  let query = $state("");

  $effect(() => {
    version;
    draft = {};
    api.bindings(account).then((b) => (base = b)).catch((e) => notify(String(e), true));
  });

  const original = $derived(Object.fromEntries(base.map((b) => [b.id, b.binding])));
  const effective = $derived({ ...original, ...draft });
  const clashing = $derived(conflicts(effective));
  const edits = $derived(Object.keys(draft).length);
  const groups = $derived.by(() => {
    const q = query.trim().toLowerCase();
    const rows = base.filter((b) => !q || [label(b.id), b.id, effective[b.id] ?? ""].some((s) => s.toLowerCase().includes(q)));
    return GROUP_ORDER.map((g) => [g, rows.filter((b) => group(b.id) === g)] as const).filter(([, r]) => r.length);
  });

  function set(id: string, value: string | null) {
    if (value === original[id]) delete draft[id];
    else draft[id] = value;
    capturing = null;
  }

  function onkeydown(e: KeyboardEvent) {
    if (!capturing) return;
    e.preventDefault();
    if (e.code === "Escape" && !e.shiftKey && !e.ctrlKey && !e.altKey) return (capturing = null);
    const k = fromKey(e);
    if (k) set(capturing, k);
  }

  function onmousedown(e: MouseEvent) {
    if (!capturing) return;
    const k = fromMouse(e);
    if (k) {
      e.preventDefault();
      set(capturing, k);
    } else if (!(e.target as HTMLElement).closest(`[data-id="${capturing}"]`)) {
      capturing = null;
    }
  }

  const onwheel = (e: WheelEvent) => capturing && set(capturing, fromWheel(e));

  async function saveFile() {
    const path = await save({ defaultPath: "my-dota-config.json", filters: [{ name: "Dota preset", extensions: ["json"] }] });
    if (!path) return;
    try {
      const p = await api.exportCurrent(account, path.split(/[\\/]/).pop()!.replace(/\.json$/i, ""));
      await api.writePreset(path, { ...p, keys: { ...p.keys, ...draft } });
      notify(`Saved to ${path}`);
    } catch (e) {
      notify(String(e), true);
    }
  }
</script>

<svelte:window {onkeydown} {onmousedown} {onwheel} />

<div class="bar">
  <input type="search" placeholder="Search actions or keys…" bind:value={query} />
  <span class="status">
    {#if capturing}<b class="hot">Press a key or mouse button · Esc cancels</b>
    {:else if edits}<b>{edits}</b> unsaved edit{edits === 1 ? "" : "s"}
    {:else}Click a key to rebind it{/if}
    {#if clashing.size}<span class="warn">· {clashing.size} actions share a key</span>{/if}
  </span>
  <button class="ghost" disabled={!edits} onclick={() => (draft = {})}>Discard</button>
  <button onclick={saveFile}>Save as file…</button>
  <button class="primary" disabled={!edits} onclick={() => onreview({ ...emptyPreset("Custom edits"), keys: { ...draft } })}>Review & apply</button>
</div>

{#each groups as [name, rows]}
  <section>
    <h3>{name} <span>{rows.length}</span></h3>
    <div class="rows">
      {#each rows as b (b.id)}
        {@const edited = b.id in draft}
        <div class="row" class:edited>
          <div class="name">
            <span>{label(b.id)}</span>
            <code>{b.id}</code>
          </div>
          <button class="cap" data-id={b.id} onclick={() => (capturing = capturing === b.id ? null : b.id)}>
            <Keycap binding={effective[b.id]} tone={capturing === b.id ? "capture" : clashing.has(b.id) ? "conflict" : edited ? "new" : ""} />
          </button>
          <div class="tools">
            {#if effective[b.id]}<button class="ghost" title="Unbind" onclick={() => set(b.id, null)}>✕</button>{/if}
            {#if edited}<button class="ghost" title="Revert" onclick={() => set(b.id, original[b.id])}>↺</button>{/if}
          </div>
        </div>
      {/each}
    </div>
  </section>
{/each}

<style>
  .bar {
    position: sticky;
    top: 0;
    z-index: 2;
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 12px 0;
    background: linear-gradient(var(--bg) 75%, transparent);
  }
  .bar input { width: 240px; }
  .status { flex: 1; font-size: 13px; color: var(--muted); }
  .status b { color: var(--text); }
  .status .hot { color: var(--ember-hi); }
  .warn { color: var(--gold); }
  section { margin-top: 18px; }
  h3 { font-size: 12px; letter-spacing: 0.2em; text-transform: uppercase; color: var(--muted); padding-bottom: 6px; border-bottom: 1px solid var(--line-2); }
  h3 span { color: var(--ember-hi); margin-left: 6px; }
  .rows { display: grid; grid-template-columns: repeat(auto-fill, minmax(380px, 1fr)); column-gap: 24px; }
  .row { display: grid; grid-template-columns: 1fr auto 64px; align-items: center; gap: 8px; padding: 5px 6px; border-bottom: 1px solid var(--line); }
  .row.edited { background: linear-gradient(90deg, rgba(111, 191, 142, 0.08), transparent); }
  .name { display: flex; flex-direction: column; min-width: 0; }
  .name code { font: 10px var(--mono); color: #5f574f; overflow: hidden; text-overflow: ellipsis; }
  .cap { all: unset; cursor: pointer; padding: 2px; }
  .cap:focus-visible { outline: 1px solid var(--ember); }
  .tools { display: flex; justify-content: flex-end; }
  .tools button { padding: 4px 8px; clip-path: none; font-size: 12px; }
</style>

<script lang="ts">
  import { open, save } from "@tauri-apps/plugin-dialog";
  import Keycap from "./Keycap.svelte";
  import { api, type Preset } from "./api";

  let { account, onreview, notify }: { account: number; onreview: (p: Preset) => void; notify: (msg: string, error?: boolean) => void } = $props();

  const FILTERS = [{ name: "Dota preset", extensions: ["json"] }];
  const ABILITIES = ["AbilityPrimary1", "AbilityPrimary2", "AbilityPrimary3", "AbilitySecondary1", "AbilitySecondary2", "AbilityUltimate"];
  const ITEMS = ["Inventory1", "Inventory2", "Inventory3", "Inventory4", "Inventory5", "Inventory6"];

  let presets = $state<Preset[]>([]);
  api.bundledPresets().then((p) => (presets = p));

  const count = (p: Preset) => Object.keys(p.keys).length + Object.keys(p.settings).length + Object.keys(p.convars).length + Object.keys(p.binds).length;

  async function importFile() {
    const path = await open({ filters: FILTERS });
    if (!path) return;
    try {
      onreview(await api.readPreset(path));
    } catch (e) {
      notify(String(e), true);
    }
  }

  async function exportCurrent() {
    const path = await save({ defaultPath: "my-dota-config.json", filters: FILTERS });
    if (!path) return;
    try {
      const name = path.split(/[\\/]/).pop()!.replace(/\.json$/i, "");
      await api.writePreset(path, await api.exportCurrent(account, name));
      notify(`Saved current setup to ${path}`);
    } catch (e) {
      notify(String(e), true);
    }
  }
</script>

<section class="actions">
  <div class="action">
    <span class="eyebrow">Share</span>
    <h3>Export my setup</h3>
    <p>Saves every hotkey and game setting to a preset file you can keep or send to a friend.</p>
    <button onclick={exportCurrent}>Export to file…</button>
  </div>
  <div class="action">
    <span class="eyebrow">Load</span>
    <h3>Import a preset</h3>
    <p>Opens a preset file and shows exactly what will change before anything is written.</p>
    <button onclick={importFile}>Import file…</button>
  </div>
</section>

<span class="eyebrow">Built-in presets</span>
<section class="grid">
  {#each presets as p, i}
    <article style="animation-delay: {i * 60}ms">
      <h3>{p.name}</h3>
      <p>{p.description}</p>
      {#if ABILITIES.some((a) => a in p.keys)}
        <div class="row"><span>Abilities</span>{#each ABILITIES as a}<Keycap binding={p.keys[a] ?? null} small />{/each}</div>
      {/if}
      {#if ITEMS.some((a) => a in p.keys)}
        <div class="row"><span>Items</span>{#each ITEMS as a}<Keycap binding={p.keys[a] ?? null} small />{/each}</div>
      {/if}
      <footer>
        <span class="meta">{count(p)} settings</span>
        <button class="primary" onclick={() => onreview(p)}>Preview</button>
      </footer>
    </article>
  {/each}
</section>

<style>
  .actions { display: grid; grid-template-columns: 1fr 1fr; gap: 14px; margin-bottom: 28px; }
  .action { padding: 18px 20px; border: 1px solid var(--line); background: linear-gradient(160deg, var(--panel-2), var(--panel)); }
  .action h3 { font-size: 18px; margin: 4px 0 6px; }
  .action p { color: var(--muted); margin: 0 0 14px; font-size: 14px; }
  .grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(290px, 1fr)); gap: 14px; margin-top: 10px; }
  article {
    display: flex;
    flex-direction: column;
    gap: 10px;
    padding: 18px 20px;
    background: var(--panel);
    border: 1px solid var(--line);
    border-left: 3px solid var(--ember);
    animation: rise 0.3s ease-out both;
  }
  article h3 { font-size: 20px; }
  article p { margin: 0; color: var(--muted); font-size: 14px; flex: 1; }
  .row { display: flex; align-items: center; gap: 4px; flex-wrap: wrap; }
  .row span { width: 66px; font: 600 10px var(--display); letter-spacing: 0.16em; text-transform: uppercase; color: var(--muted); }
  footer { display: flex; align-items: center; justify-content: space-between; margin-top: 4px; }
  .meta { font: 12px var(--mono); color: var(--muted); }
</style>

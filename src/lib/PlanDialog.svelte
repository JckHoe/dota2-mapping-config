<script lang="ts">
  import Keycap from "./Keycap.svelte";
  import { label } from "./keys";
  import type { Plan, Preset } from "./api";

  let { preset, plan, busy, dotaRunning, onconfirm, oncancel }: {
    preset: Preset; plan: Plan; busy: boolean; dotaRunning: boolean; onconfirm: () => void; oncancel: () => void;
  } = $props();

  const SECTIONS = { key: "Hotkeys", setting: "Hotkey options", convar: "Game settings", bind: "Console binds" } as const;
  const grouped = $derived(
    (Object.keys(SECTIONS) as (keyof typeof SECTIONS)[])
      .map((s) => [s, plan.changes.filter((c) => c.section === s)] as const)
      .filter(([, cs]) => cs.length),
  );
</script>

<svelte:window onkeydown={(e) => e.key === "Escape" && !busy && oncancel()} />

<div class="wrap">
  <div class="backdrop" role="presentation" onclick={oncancel}></div>
  <div class="dialog" role="dialog" aria-modal="true">
    <header>
      <span class="eyebrow">Review changes</span>
      <h2>{preset.name}</h2>
    </header>

    <div class="body">
      {#if plan.warnings.length}
        <ul class="warnings">
          {#each plan.warnings as w}<li>{w}</li>{/each}
        </ul>
      {/if}

      {#if !plan.changes.length}
        <p class="empty">Nothing to change. Your config already matches this preset.</p>
      {/if}

      {#each grouped as [section, changes]}
        <h3>{SECTIONS[section]} <span>{changes.length}</span></h3>
        <table>
          <tbody>
            {#each changes as c}
              <tr>
                <td class="id">{section === "key" ? label(c.id) : c.id}</td>
                {#if section === "key"}
                  <td><Keycap binding={c.from} tone="old" small /></td>
                  <td class="arrow">→</td>
                  <td><Keycap binding={c.to} tone="new" small /></td>
                {:else}
                  <td class="val old">{c.from ?? "—"}</td>
                  <td class="arrow">→</td>
                  <td class="val">{c.to ?? "unbound"}</td>
                {/if}
              </tr>
            {/each}
          </tbody>
        </table>
      {/each}
    </div>

    <footer>
      <p>{dotaRunning ? "Close Dota 2 to apply." : "A backup is saved first. If Steam reports a cloud conflict on next launch, keep the local files."}</p>
      <button class="ghost" onclick={oncancel}>Cancel</button>
      <button class="primary" disabled={busy || dotaRunning || !plan.changes.length} onclick={onconfirm}>
        {busy ? "Applying…" : `Apply ${plan.changes.length} change${plan.changes.length === 1 ? "" : "s"}`}
      </button>
    </footer>
  </div>
</div>

<style>
  .wrap { position: fixed; inset: 0; display: grid; place-items: center; z-index: 10; }
  .backdrop { position: absolute; inset: 0; background: rgba(5, 4, 4, 0.72); backdrop-filter: blur(3px); }
  .dialog {
    position: relative;
    width: min(680px, calc(100vw - 32px));
    max-height: calc(100vh - 64px);
    display: flex;
    flex-direction: column;
    background: var(--panel);
    border: 1px solid var(--line-2);
    border-top: 2px solid var(--ember);
    animation: rise 0.18s ease-out;
  }
  header { padding: 18px 22px 12px; border-bottom: 1px solid var(--line); }
  header h2 { font-size: 22px; margin-top: 2px; }
  .body { padding: 6px 22px 16px; overflow: auto; }
  h3 { font-size: 13px; letter-spacing: 0.14em; text-transform: uppercase; color: var(--muted); margin: 18px 0 8px; }
  h3 span { color: var(--ember-hi); margin-left: 6px; }
  table { width: 100%; border-collapse: collapse; table-layout: fixed; }
  td { padding: 5px 0; border-bottom: 1px solid var(--line); vertical-align: middle; }
  td.id { width: 44%; color: var(--text); overflow: hidden; text-overflow: ellipsis; }
  td:nth-child(2) { width: 24%; }
  td.arrow { color: var(--muted); width: 32px; text-align: center; }
  td.val { font: 12px var(--mono); word-break: break-all; }
  td.val.old { color: var(--muted); }
  .warnings { list-style: none; margin: 14px 0 0; padding: 10px 12px; border-left: 3px solid var(--gold); background: rgba(224, 168, 74, 0.08); color: #f1d19a; font-size: 13px; }
  .warnings li + li { margin-top: 4px; }
  .empty { color: var(--muted); margin-top: 18px; }
  footer { display: flex; align-items: center; gap: 8px; padding: 14px 22px; border-top: 1px solid var(--line); }
  footer p { flex: 1; margin: 0; font-size: 12px; color: var(--muted); }
</style>

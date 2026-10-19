<script lang="ts">
  import { confirm } from "@tauri-apps/plugin-dialog";
  import { api, type Snapshot } from "./api";

  let { account, dotaRunning, notify }: { account: number; dotaRunning: boolean; notify: (msg: string, error?: boolean) => void } = $props();

  let snaps = $state<Snapshot[]>([]);
  let label = $state("");
  let busy = $state(false);

  const load = () => api.snapshots(account).then((s) => (snaps = s)).catch((e) => notify(String(e), true));
  $effect(() => {
    account;
    load();
  });

  async function run(fn: () => Promise<unknown>, msg: string) {
    busy = true;
    try {
      await fn();
      notify(msg);
    } catch (e) {
      notify(String(e), true);
    }
    busy = false;
    load();
  }

  const create = () => run(() => api.createSnapshot(account, label.trim() || "Manual backup").then(() => (label = "")), "Backup saved");
  const restore = async (s: Snapshot) => (await confirm(`Restore "${s.label}"? Your current files are backed up first.`)) && run(() => api.restoreSnapshot(account, s.id), `Restored "${s.label}"`);
  const remove = async (s: Snapshot) => (await confirm(`Delete backup "${s.label}"?`)) && run(() => api.deleteSnapshot(account, s.id), "Backup deleted");
  const when = (ms: number) => new Date(ms).toLocaleString(undefined, { dateStyle: "medium", timeStyle: "short" });
</script>

<form class="new" onsubmit={(e) => (e.preventDefault(), create())}>
  <div>
    <span class="eyebrow">Full backup</span>
    <p>Exact copy of your hotkey, settings and bind files. Restore puts them back byte-for-byte.</p>
  </div>
  <input placeholder="Label, e.g. before patch 7.40" bind:value={label} />
  <button class="primary" disabled={busy}>Back up now</button>
</form>

{#if !snaps.length}
  <p class="empty">No backups yet. One is made automatically every time you apply a preset.</p>
{/if}

<ol>
  {#each snaps as s (s.id)}
    <li>
      <time>{when(s.created_ms)}</time>
      <span class="label">{s.label}</span>
      <button disabled={busy || dotaRunning} title={dotaRunning ? "Close Dota 2 first" : ""} onclick={() => restore(s)}>Restore</button>
      <button class="ghost" disabled={busy} onclick={() => remove(s)}>Delete</button>
    </li>
  {/each}
</ol>

<style>
  .new { display: grid; grid-template-columns: 1fr 260px auto; gap: 12px; align-items: center; padding: 18px 20px; border: 1px solid var(--line); background: var(--panel); margin-bottom: 18px; }
  .new p { margin: 4px 0 0; color: var(--muted); font-size: 14px; }
  .empty { color: var(--muted); }
  ol { list-style: none; padding: 0; margin: 0; border-left: 1px solid var(--line-2); }
  li { position: relative; display: grid; grid-template-columns: 180px 1fr auto auto; gap: 10px; align-items: center; padding: 10px 0 10px 22px; animation: rise 0.25s ease-out both; }
  li::before { content: ""; position: absolute; left: -5px; width: 9px; height: 9px; background: var(--panel); border: 2px solid var(--ember); transform: rotate(45deg); }
  li + li { border-top: 1px solid var(--line); }
  time { font: 12px var(--mono); color: var(--muted); }
  .label { font-weight: 500; }
</style>

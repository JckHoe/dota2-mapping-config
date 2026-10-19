<script lang="ts">
  import Backups from "$lib/Backups.svelte";
  import Editor from "$lib/Editor.svelte";
  import PlanDialog from "$lib/PlanDialog.svelte";
  import Presets from "$lib/Presets.svelte";
  import { api, type Account, type Plan, type Preset } from "$lib/api";

  const TABS = ["Presets", "Key editor", "Backups"] as const;

  let tab = $state<(typeof TABS)[number]>("Presets");
  let accounts = $state<Account[]>([]);
  let account = $state<number | null>(null);
  let loadError = $state("");
  let dotaRunning = $state(false);
  let version = $state(0);
  let review = $state<{ preset: Preset; plan: Plan } | null>(null);
  let busy = $state(false);
  let toast = $state<{ msg: string; error: boolean } | null>(null);
  let toastTimer: ReturnType<typeof setTimeout>;

  api.accounts().then((a) => ((accounts = a), (account = a[0]?.id ?? null))).catch((e) => (loadError = String(e)));

  $effect(() => {
    const poll = () => api.dotaRunning().then((r) => (dotaRunning = r));
    poll();
    const t = setInterval(poll, 3000);
    return () => clearInterval(t);
  });

  function notify(msg: string, error = false) {
    toast = { msg, error };
    clearTimeout(toastTimer);
    toastTimer = setTimeout(() => (toast = null), error ? 8000 : 4000);
  }

  async function openReview(preset: Preset) {
    try {
      review = { preset, plan: await api.preview(account!, preset) };
    } catch (e) {
      notify(String(e), true);
    }
  }

  async function confirmApply() {
    if (!review) return;
    busy = true;
    try {
      const plan = await api.apply(account!, review.preset);
      notify(`Applied ${plan.changes.length} changes from ${review.preset.name}. Backup saved.`);
      review = null;
      version++;
    } catch (e) {
      notify(String(e), true);
    }
    busy = false;
  }
</script>

<div class="shell">
  <header>
    <div class="brand">
      <span class="mark">◆</span>
      <div>
        <h1>Dota Config</h1>
        <span class="eyebrow">Hotkeys · settings · presets</span>
      </div>
    </div>
    <nav>
      {#each TABS as t}
        <button class="tab" class:active={tab === t} onclick={() => (tab = t)}>{t}</button>
      {/each}
    </nav>
    <div class="right">
      <span class="pill" class:live={dotaRunning} title={dotaRunning ? "Close Dota 2 before applying changes" : ""}>
        <i></i>{dotaRunning ? "Dota 2 running" : "Dota 2 closed"}
      </span>
      {#if accounts.length > 1}
        <select bind:value={account}>
          {#each accounts as a}<option value={a.id}>{a.name ?? a.id}</option>{/each}
        </select>
      {:else if accounts[0]}
        <span class="who">{accounts[0].name ?? accounts[0].id}</span>
      {/if}
    </div>
  </header>

  <main>
    {#if loadError}
      <div class="error"><h2>Couldn't find Dota 2</h2><p>{loadError}</p><p>Launch Dota 2 once on this computer so it creates its config files, then reopen this app.</p></div>
    {:else if account !== null}
      {#key account}
        {#if tab === "Presets"}
          <Presets {account} onreview={openReview} {notify} />
        {:else if tab === "Key editor"}
          <Editor {account} {version} onreview={openReview} {notify} />
        {:else}
          <Backups {account} {dotaRunning} {notify} />
        {/if}
      {/key}
    {/if}
  </main>
</div>

{#if review}
  <PlanDialog preset={review.preset} plan={review.plan} {busy} {dotaRunning} onconfirm={confirmApply} oncancel={() => (review = null)} />
{/if}

{#if toast}
  <div class="toast" class:error={toast.error} role="status">{toast.msg}</div>
{/if}

<style>
  .shell { height: 100vh; display: flex; flex-direction: column; }
  header {
    display: grid;
    grid-template-columns: auto 1fr auto;
    align-items: center;
    gap: 28px;
    padding: 14px 28px;
    border-bottom: 1px solid var(--line);
    background: rgba(15, 13, 12, 0.7);
  }
  .brand { display: flex; align-items: center; gap: 12px; }
  .mark { font-size: 26px; color: var(--ember); text-shadow: 0 0 18px rgba(208, 69, 47, 0.6); }
  h1 { font-size: 20px; text-transform: uppercase; letter-spacing: 0.12em; line-height: 1; }
  nav { display: flex; gap: 2px; }
  .tab { background: transparent; border: none; clip-path: none; color: var(--muted); padding: 10px 14px; border-bottom: 2px solid transparent; }
  .tab:hover:not(:disabled) { background: transparent; color: var(--text); }
  .tab.active { color: var(--text); border-bottom-color: var(--ember); }
  .right { display: flex; align-items: center; gap: 12px; }
  .who { font-weight: 500; }
  .pill { display: inline-flex; align-items: center; gap: 7px; font: 600 11px var(--display); letter-spacing: 0.12em; text-transform: uppercase; color: var(--muted); }
  .pill i { width: 8px; height: 8px; border-radius: 50%; background: var(--jade); }
  .pill.live { color: var(--gold); }
  .pill.live i { background: var(--gold); box-shadow: 0 0 10px var(--gold); }
  main { flex: 1; overflow: auto; padding: 22px 28px 40px; }
  .error { max-width: 560px; padding: 22px; border: 1px solid var(--line-2); border-left: 3px solid var(--ember); background: var(--panel); }
  .error p { color: var(--muted); }
  .toast {
    position: fixed;
    bottom: 20px;
    left: 50%;
    transform: translateX(-50%);
    max-width: 640px;
    padding: 10px 16px;
    background: var(--panel-2);
    border: 1px solid var(--line-2);
    border-left: 3px solid var(--jade);
    animation: rise 0.2s ease-out;
    z-index: 20;
  }
  .toast.error { border-left-color: var(--ember); }
</style>

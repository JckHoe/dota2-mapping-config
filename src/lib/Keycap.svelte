<script lang="ts">
  const SHORT: Record<string, string> = {
    UPARROW: "↑", DOWNARROW: "↓", LEFTARROW: "←", RIGHTARROW: "→", SPACE: "Space", ENTER: "Enter", KP_ENTER: "Num Enter",
    BACKSPACE: "Bksp", ESCAPE: "Esc", SEMICOLON: ";", MOUSE3: "Mouse 3", MOUSE4: "Mouse 4", MOUSE5: "Mouse 5",
    MWHEELUP: "Wheel ↑", MWHEELDOWN: "Wheel ↓", CAPSLOCK: "Caps",
  };

  let { binding, tone = "", small = false }: { binding: string | null; tone?: string; small?: boolean } = $props();

  const parts = $derived(binding ? (/^(SHIFT|CTRL|ALT)\+(.+)$/.exec(binding)?.slice(1) ?? [binding]) : []);
</script>

<span class="caps {tone}" class:small>
  {#each parts as p, i}
    {#if i > 0}<span class="plus">+</span>{/if}
    <kbd class:wide={(SHORT[p] ?? p).length > 2}>{SHORT[p] ?? p}</kbd>
  {:else}
    <kbd class="none">—</kbd>
  {/each}
</span>

<style>
  .caps { display: inline-flex; align-items: center; gap: 4px; }
  kbd {
    min-width: 30px;
    height: 30px;
    padding: 0 7px;
    display: inline-grid;
    place-items: center;
    font: 500 12px var(--mono);
    color: var(--text);
    background: linear-gradient(#2b2522, #1d1917);
    border: 1px solid #453b36;
    border-bottom-width: 3px;
    border-radius: 5px;
    box-shadow: 0 2px 0 rgba(0, 0, 0, 0.5), inset 0 1px 0 rgba(255, 255, 255, 0.06);
    white-space: nowrap;
  }
  .small kbd { min-width: 24px; height: 24px; font-size: 11px; padding: 0 5px; }
  kbd.none { color: var(--muted); background: transparent; border-style: dashed; border-bottom-width: 1px; box-shadow: none; }
  .plus { color: var(--muted); font: 11px var(--mono); }
  .new kbd:not(.none) { border-color: var(--jade); color: #d9f5e3; background: linear-gradient(#24382c, #182419); }
  .old kbd { opacity: 0.55; text-decoration: line-through; text-decoration-color: var(--muted); }
  .conflict kbd:not(.none) { border-color: var(--gold); color: #ffe4b0; }
  .capture kbd { border-color: var(--ember-hi); color: var(--ember-hi); animation: pulse 1s ease-in-out infinite; }
  @keyframes pulse { 50% { box-shadow: 0 0 0 3px rgba(255, 106, 74, 0.25); } }
</style>

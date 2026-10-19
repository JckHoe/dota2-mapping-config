const CODES: Record<string, string> = {
  Space: "SPACE", Enter: "ENTER", NumpadEnter: "KP_ENTER", Tab: "TAB", Backspace: "BACKSPACE", Escape: "ESCAPE",
  ArrowUp: "UPARROW", ArrowDown: "DOWNARROW", ArrowLeft: "LEFTARROW", ArrowRight: "RIGHTARROW",
  Semicolon: "SEMICOLON", Quote: "'", Backslash: "\\", BracketLeft: "[", BracketRight: "]", Minus: "-", Equal: "=",
  Comma: ",", Period: ".", Slash: "/", Backquote: "`", Insert: "INS", Delete: "DEL", Home: "HOME", End: "END",
  PageUp: "PGUP", PageDown: "PGDN", CapsLock: "CAPSLOCK",
  NumpadDivide: "KP_DIVIDE", NumpadMultiply: "KP_MULTIPLY", NumpadSubtract: "KP_MINUS", NumpadAdd: "KP_PLUS", NumpadDecimal: "KP_DEL",
};
const MOUSE: Record<number, string> = { 1: "MOUSE3", 3: "MOUSE4", 4: "MOUSE5" };
const MODIFIER_CODES = ["ShiftLeft", "ShiftRight", "ControlLeft", "ControlRight", "AltLeft", "AltRight", "MetaLeft", "MetaRight"];

export const OTHER_CONTEXTS = ["Spectator", "ShopTab", "ShopSlot", "Legacy", "TalentUpgrade", "NeutralItemSelect"];

function withModifier(e: KeyboardEvent | MouseEvent, key: string) {
  const mod = e.ctrlKey ? "CTRL" : e.altKey ? "ALT" : e.shiftKey ? "SHIFT" : "";
  return mod ? `${mod}+${key}` : key;
}

export function fromKey(e: KeyboardEvent): string | null {
  if (MODIFIER_CODES.includes(e.code)) return null;
  const c = e.code;
  const key = c.startsWith("Key") ? c.slice(3)
    : c.startsWith("Digit") ? c.slice(5)
    : /^F\d+$/.test(c) ? c
    : /^Numpad\d$/.test(c) ? `KP_${c.slice(6)}`
    : CODES[c];
  return key ? withModifier(e, key) : null;
}

export const fromMouse = (e: MouseEvent) => (MOUSE[e.button] ? withModifier(e, MOUSE[e.button]) : null);
export const fromWheel = (e: WheelEvent) => withModifier(e, e.deltaY < 0 ? "MWHEELUP" : "MWHEELDOWN");

const ABILITY: Record<string, string> = {
  Primary1: "Ability 1", Primary2: "Ability 2", Primary3: "Ability 3", Secondary1: "Ability 4", Secondary2: "Ability 5", Ultimate: "Ultimate",
};
const LABELS: Record<string, string> = {
  InventoryTP: "TP scroll", InventoryTPAuto: "TP scroll (autocast)", InventoryTPAutoCast: "TP scroll (autocast)", UseActionItem: "Neutral item",
  HeroAttack: "Attack", HeroMove: "Move", HeroStop: "Stop", HeroHold: "Hold position", HeroPatrol: "Patrol", HeroSelect: "Select hero",
  CourierSelect: "Select courier", CourierDeliver: "Courier deliver", CourierHaste: "Courier speed burst", SelectAllOthers: "Select other units",
  SelectAll: "Select all units", ActivateRadar: "Scan", ActivateGlyph: "Glyph", ShopToggle: "Shop", PurchaseQuickbuy: "Buy quickbuy",
  PurchaseSticky: "Buy sticky item", GrabStashItems: "Grab stash", LearnAbilities: "Learn ability mode", LearnStats: "Learn talent",
  ScoreboardToggle: "Scoreboard", CameraGrip: "Camera drag", HeroTaunt: "Taunt", ChatTeam: "Team chat", ChatGlobal: "All chat",
  ChatTeam2: "Team chat (alt)", ChatGlobal2: "All chat (alt)", ChatVoice: "Voice", ChatWheel: "Chat wheel", ControlGroupCycle: "Cycle units",
  RecentEvent: "Jump to last event", InspectHeroInWorld: "Inspect hero",
};

export function label(id: string): string {
  if (LABELS[id]) return LABELS[id];
  const ab = id.match(/^(Legacy)?Ability(Primary\d|Secondary\d|Ultimate)(AutoCast)?$/);
  if (ab) return `${ab[1] ? "Legacy " : ""}${ABILITY[ab[2]]}${ab[3] ? " (autocast)" : ""}`;
  const inv = id.match(/^Inventory(\d)(QuickCast|AutoCast)?$/);
  if (inv) return `Item slot ${inv[1]}${inv[2] ? ` (${inv[2] === "QuickCast" ? "quick cast" : "autocast"})` : ""}`;
  return id.replace(/([a-z])([A-Z0-9])/g, "$1 $2").replace(/_/g, " ");
}

const GROUPS: [string, RegExp][] = [
  ["Abilities", /^(Ability|Learn|TalentUpgrade)/],
  ["Items", /^(Inventory|UseActionItem|NeutralItem)/],
  ["Units", /^(Hero(?!ChatWheel)|Courier|Select|ControlGroup|Activate)/],
  ["Camera", /^Camera/],
  ["Shop", /^(Shop|Purchase|GrabStash)/],
  ["Chat", /^(Chat|HeroChatWheel)/],
  ["Legacy keys", /^Legacy/],
  ["Spectator", /^Spectator/],
];

export const GROUP_ORDER = [...GROUPS.map(([g]) => g), "Other"];
export const group = (id: string) => GROUPS.find(([, re]) => re.test(id))?.[0] ?? "Other";

export function conflicts(bindings: Record<string, string | null>): Set<string> {
  const byKey = new Map<string, string[]>();
  for (const [id, b] of Object.entries(bindings)) {
    if (!b || OTHER_CONTEXTS.some((c) => id.startsWith(c))) continue;
    byKey.set(b, [...(byKey.get(b) ?? []), id]);
  }
  return new Set([...byKey.values()].filter((ids) => ids.length > 1).flat());
}

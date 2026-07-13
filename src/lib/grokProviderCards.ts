/**
 * Grok 登录方式卡片合成与排序。
 *
 * 用例表（实现契约，可人工核对）：
 *
 * 1) 默认序（无 order / 无 pin）
 *    profiles = [{id:"a"},{id:"b"}]
 *    → keys: official, profile:a, profile:b
 *
 * 2) 置顶优先
 *    pinned = ["profile:b"], order = []
 *    → profile:b, official, profile:a
 *
 * 3) 死 key 过滤 + order 保序
 *    order = ["profile:gone","profile:b","official"], profiles=[{id:"a"},{id:"b"}]
 *    → profile:b, official, profile:a
 *    （gone 丢弃；a 未出现则按 profiles 原序追加）
 */

import type { GrokProfile, GrokProviderStatus } from "../types";

export const OFFICIAL_PROVIDER_KEY = "official";

export type ProviderCardKind = "official" | "profile";

export type ProviderCard = {
  key: string;
  kind: ProviderCardKind;
  name: string;
  isActive: boolean;
  pinned: boolean;
  /** profile only */
  profile?: GrokProfile;
  /** official only */
  loggedIn?: boolean;
  subtitle: string;
  meta: string;
};

export type ProviderLayoutInput = {
  order?: string[];
  pinnedIds?: string[];
};

export function profileProviderKey(id: string): string {
  return `profile:${id}`;
}

export function buildProviderCards(
  profiles: GrokProfile[],
  status: GrokProviderStatus | null,
  layout: ProviderLayoutInput = {},
): ProviderCard[] {
  const officialActive = status?.officialActive ?? !profiles.some((p) => p.isActive);
  const loggedIn = status?.officialLoggedIn ?? false;

  const candidates: ProviderCard[] = [
    {
      key: OFFICIAL_PROVIDER_KEY,
      kind: "official",
      name: "官方账号",
      isActive: officialActive,
      pinned: false,
      loggedIn,
      subtitle: "grok.com / auth.json",
      meta: `${loggedIn ? "已登录 grok.com" : "尚未登录"} · OAuth 官方模型`,
    },
    ...profiles.map((profile) => ({
      key: profileProviderKey(profile.id),
      kind: "profile" as const,
      name: profile.name,
      isActive: profile.isActive,
      pinned: false,
      profile,
      subtitle: profile.baseUrl || "（无 Base URL）",
      meta: `${profile.defaultModel || "未设默认模型"} · ${profile.models?.length ?? 0} 模型`,
    })),
  ];

  const candidateKeys = new Set(candidates.map((c) => c.key));
  const orderPref = uniqueNonEmpty(layout.order ?? []).filter((k) => candidateKeys.has(k));
  // 1) 偏好 order 保序；2) official 若缺失插最前；3) 其余按 candidates 原序追加
  const effectiveOrder = [...orderPref];
  if (!effectiveOrder.includes(OFFICIAL_PROVIDER_KEY)) {
    effectiveOrder.unshift(OFFICIAL_PROVIDER_KEY);
  }
  for (const card of candidates) {
    if (!effectiveOrder.includes(card.key)) {
      effectiveOrder.push(card.key);
    }
  }

  const pinned = new Set(
    uniqueNonEmpty(layout.pinnedIds ?? []).filter((k) => candidateKeys.has(k)),
  );
  const position = new Map(effectiveOrder.map((key, index) => [key, index]));

  return candidates
    .map((card) => ({
      ...card,
      pinned: pinned.has(card.key),
    }))
    .sort((a, b) => {
      const pinDiff = Number(b.pinned) - Number(a.pinned);
      if (pinDiff !== 0) return pinDiff;
      return (position.get(a.key) ?? 0) - (position.get(b.key) ?? 0);
    });
}

export function filterProviderCards(cards: ProviderCard[], query: string): ProviderCard[] {
  const q = query.trim().toLowerCase();
  if (!q) return cards;
  return cards.filter((card) => {
    if (card.kind === "official") {
      const haystack = `${card.name} ${card.subtitle} auth.json grok.com 官方`.toLowerCase();
      return haystack.includes(q);
    }
    const p = card.profile;
    if (!p) return card.name.toLowerCase().includes(q);
    return (
      p.name.toLowerCase().includes(q) ||
      p.baseUrl.toLowerCase().includes(q) ||
      p.defaultModel.toLowerCase().includes(q)
    );
  });
}

function uniqueNonEmpty(items: string[]): string[] {
  const seen = new Set<string>();
  const out: string[] = [];
  for (const item of items) {
    const s = item.trim();
    if (!s || seen.has(s)) continue;
    seen.add(s);
    out.push(s);
  }
  return out;
}

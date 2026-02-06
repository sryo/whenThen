// RSS sources and interests state management.

import { invoke } from "@tauri-apps/api/core";

export interface Source {
  id: string;
  name: string;
  url: string;
  enabled: boolean;
  lastChecked?: string;
}

export interface Interest {
  id: string;
  name: string;
  enabled: boolean;
  filters: FeedFilter[];
  filterLogic: "and" | "or";
}

export interface FeedFilter {
  type: "must_contain" | "must_not_contain" | "regex" | "size_range";
  value: string;
  enabled: boolean;
}

interface FeedTestResult {
  items: FeedTestItem[];
  totalCount: number;
  matchedCount: number;
}

interface FeedTestItem {
  title: string;
  matches: boolean;
  matchedFilter?: string;
  size?: number;
}

export interface PendingMatch {
  id: string;
  sourceId: string;
  sourceName: string;
  interestId: string;
  interestName: string;
  title: string;
  magnetUri?: string;
  torrentUrl?: string;
  createdAt: string;
  metadata?: TorrentMetadata;
}

interface TorrentMetadata {
  name: string;
  totalSize: number;
  fileCount: number;
  files: TorrentFilePreview[];
}

interface TorrentFilePreview {
  name: string;
  size: number;
  isVideo: boolean;
  isSuspicious: boolean;
}

export interface BadItem {
  infoHash: string;
  title: string;
  interestId?: string;
  interestName?: string;
  markedAt: string;
  reason?: string;
}

interface TorrentInterestLink {
  interestId: string;
  interestName: string;
}

// Convert from Rust snake_case to JS camelCase
function sourceFromRust(s: any): Source {
  return {
    id: s.id,
    name: s.name,
    url: s.url,
    enabled: s.enabled,
    lastChecked: s.last_checked,
  };
}

function sourceToRust(s: Source): any {
  return {
    id: s.id,
    name: s.name,
    url: s.url,
    enabled: s.enabled,
    last_checked: s.lastChecked,
  };
}

function interestFromRust(i: any): Interest {
  return {
    id: i.id,
    name: i.name,
    enabled: i.enabled,
    filters: i.filters.map((f: any) => ({
      type: f.type,
      value: f.value,
      enabled: f.enabled,
    })),
    filterLogic: i.filter_logic || "and",
  };
}

function interestToRust(i: Interest): any {
  return {
    id: i.id,
    name: i.name,
    enabled: i.enabled,
    filters: i.filters.map((f) => ({
      type: f.type,
      value: f.value,
      enabled: f.enabled,
    })),
    filter_logic: i.filterLogic,
  };
}

function pendingFromRust(p: any): PendingMatch {
  return {
    id: p.id,
    sourceId: p.source_id,
    sourceName: p.source_name,
    interestId: p.interest_id,
    interestName: p.interest_name,
    title: p.title,
    magnetUri: p.magnet_uri,
    torrentUrl: p.torrent_url,
    createdAt: p.created_at,
    metadata: p.metadata ? metadataFromRust(p.metadata) : undefined,
  };
}

function metadataFromRust(m: any): TorrentMetadata {
  return {
    name: m.name,
    totalSize: m.total_size,
    fileCount: m.file_count,
    files: m.files.map((f: any) => ({
      name: f.name,
      size: f.size,
      isVideo: f.is_video,
      isSuspicious: f.is_suspicious,
    })),
  };
}

function badItemFromRust(b: any): BadItem {
  return {
    infoHash: b.info_hash,
    title: b.title,
    interestId: b.interest_id,
    interestName: b.interest_name,
    markedAt: b.marked_at,
    reason: b.reason,
  };
}

class FeedsState {
  sources = $state<Source[]>([]);
  interests = $state<Interest[]>([]);
  pendingMatches = $state<PendingMatch[]>([]);
  // Track which interest each torrent came from (torrentId -> interestInfo)
  torrentInterests = $state<Map<number, TorrentInterestLink>>(new Map());

  get enabledSources() {
    return this.sources.filter((s) => s.enabled);
  }

  get enabledInterests() {
    return this.interests.filter((i) => i.enabled);
  }

  get pendingCount() {
    return this.pendingMatches.length;
  }

  // Source operations
  async loadSources() {
    try {
      const result: any[] = await invoke("rss_list_sources");
      this.sources = result.map(sourceFromRust);
    } catch (e) {
      console.error("Failed to load sources:", e);
    }
  }

  async addSource(source: Omit<Source, "id">) {
    const newSource: Source = {
      ...source,
      id: crypto.randomUUID(),
    };

    try {
      await invoke("rss_add_source", { source: sourceToRust(newSource) });
      this.sources = [...this.sources, newSource];
      return newSource;
    } catch (e) {
      console.error("Failed to add source:", e);
      throw e;
    }
  }

  async updateSource(id: string, updates: Partial<Source>) {
    const index = this.sources.findIndex((s) => s.id === id);
    if (index < 0) return;

    const updated = { ...this.sources[index], ...updates };

    try {
      await invoke("rss_update_source", { source: sourceToRust(updated) });
      this.sources[index] = updated;
    } catch (e) {
      console.error("Failed to update source:", e);
      throw e;
    }
  }

  async removeSource(id: string) {
    try {
      await invoke("rss_remove_source", { sourceId: id });
      this.sources = this.sources.filter((s) => s.id !== id);
    } catch (e) {
      console.error("Failed to remove source:", e);
      throw e;
    }
  }

  async toggleSource(id: string, enabled: boolean) {
    const index = this.sources.findIndex((s) => s.id === id);
    if (index < 0) return;

    try {
      await invoke("rss_toggle_source", { sourceId: id, enabled });
      this.sources[index] = { ...this.sources[index], enabled };
    } catch (e) {
      console.error("Failed to toggle source:", e);
      throw e;
    }
  }

  // Interest operations
  async loadInterests() {
    try {
      const result: any[] = await invoke("rss_list_interests");
      this.interests = result.map(interestFromRust);
    } catch (e) {
      console.error("Failed to load interests:", e);
    }
  }

  async addInterest(interest: Omit<Interest, "id" | "filterLogic"> & { filterLogic?: "and" | "or" }) {
    const newInterest: Interest = {
      ...interest,
      id: crypto.randomUUID(),
      filterLogic: interest.filterLogic || "and",
    };

    try {
      await invoke("rss_add_interest", { interest: interestToRust(newInterest) });
      this.interests = [...this.interests, newInterest];
      return newInterest;
    } catch (e) {
      console.error("Failed to add interest:", e);
      throw e;
    }
  }

  async updateInterest(id: string, updates: Partial<Interest>) {
    const index = this.interests.findIndex((i) => i.id === id);
    if (index < 0) return;

    const updated = { ...this.interests[index], ...updates };

    try {
      await invoke("rss_update_interest", { interest: interestToRust(updated) });
      this.interests[index] = updated;
    } catch (e) {
      console.error("Failed to update interest:", e);
      throw e;
    }
  }

  async removeInterest(id: string) {
    try {
      await invoke("rss_remove_interest", { interestId: id });
      this.interests = this.interests.filter((i) => i.id !== id);
    } catch (e) {
      console.error("Failed to remove interest:", e);
      throw e;
    }
  }

  async toggleInterest(id: string, enabled: boolean) {
    const index = this.interests.findIndex((i) => i.id === id);
    if (index < 0) return;

    try {
      await invoke("rss_toggle_interest", { interestId: id, enabled });
      this.interests[index] = { ...this.interests[index], enabled };
    } catch (e) {
      console.error("Failed to toggle interest:", e);
      throw e;
    }
  }

  async testInterest(url: string, filters: FeedFilter[]): Promise<FeedTestResult> {
    const result: any = await invoke("rss_test_interest", {
      url,
      filters: filters.map((f) => ({
        type: f.type,
        value: f.value,
        enabled: f.enabled,
      })),
    });

    return {
      items: result.items.map((item: any) => ({
        title: item.title,
        matches: item.matches,
        matchedFilter: item.matched_filter,
        size: item.size,
      })),
      totalCount: result.total_count,
      matchedCount: result.matched_count,
    };
  }

  // Pending matches operations
  async loadPending() {
    try {
      const result: any[] = await invoke("rss_list_pending");
      this.pendingMatches = result.map(pendingFromRust);
    } catch (e) {
      console.error("Failed to load pending matches:", e);
    }
  }

  async checkFeedsNow(): Promise<number> {
    const matched: number = await invoke("rss_check_now");
    await this.loadPending();
    return matched;
  }

  async fetchMetadata(matchId: string): Promise<TorrentMetadata> {
    const result: any = await invoke("rss_fetch_metadata", { matchId });
    const metadata = metadataFromRust(result);

    const index = this.pendingMatches.findIndex((m) => m.id === matchId);
    if (index >= 0) {
      this.pendingMatches[index] = { ...this.pendingMatches[index], metadata };
    }

    return metadata;
  }

  async approveMatch(matchId: string): Promise<number> {
    const match = this.pendingMatches.find((m) => m.id === matchId);
    const torrentId: number = await invoke("rss_approve_match", { matchId });

    // Track which interest this torrent came from
    if (match) {
      this.torrentInterests.set(torrentId, {
        interestId: match.interestId,
        interestName: match.interestName,
      });
    }

    this.pendingMatches = this.pendingMatches.filter((m) => m.id !== matchId);
    return torrentId;
  }

  getTorrentInterest(torrentId: number): TorrentInterestLink | undefined {
    return this.torrentInterests.get(torrentId);
  }

  clearTorrentInterest(torrentId: number) {
    this.torrentInterests.delete(torrentId);
  }

  async rejectMatch(matchId: string): Promise<void> {
    await invoke("rss_reject_match", { matchId });
    this.pendingMatches = this.pendingMatches.filter((m) => m.id !== matchId);
  }

  async rejectAllMatches(): Promise<void> {
    const ids = this.pendingMatches.map((m) => m.id);
    for (const id of ids) {
      await invoke("rss_reject_match", { matchId: id });
    }
    this.pendingMatches = [];
  }

  updatePendingCount(count: number) {
    if (count > this.pendingMatches.length) {
      this.loadPending();
    }
  }

  addPendingMatch(match: PendingMatch) {
    if (!this.pendingMatches.find((m) => m.id === match.id)) {
      this.pendingMatches = [...this.pendingMatches, match];
    }
  }

  // Legacy compatibility
  get feeds() {
    return this.sources;
  }

  async loadFeeds() {
    await this.loadSources();
    await this.loadInterests();
  }

  // Bad items operations
  async markBad(
    infoHash: string,
    title: string,
    interestId?: string,
    interestName?: string,
    reason?: string,
    triggerRescan?: boolean
  ): Promise<number> {
    return invoke("rss_mark_bad", {
      infoHash,
      title,
      interestId,
      interestName,
      reason,
      triggerRescan: triggerRescan ?? false,
    });
  }

  async unmarkBad(infoHash: string): Promise<void> {
    await invoke("rss_unmark_bad", { infoHash });
  }

  async listBad(): Promise<BadItem[]> {
    const result: any[] = await invoke("rss_list_bad");
    return result.map(badItemFromRust);
  }
}

export const feedsState = new FeedsState();

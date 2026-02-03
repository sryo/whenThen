// Clean torrent filenames by stripping site prefixes, tags, and codec tokens.

const SITE_PREFIX = /^www\.\S+\.\w+\s*[-–—]\s*/i;

const BRACKETED_TAGS = /\[([^\]]*)\]/g;

// Tokens to strip — order doesn't matter, matched case-insensitively as whole words.
const STRIP_TOKENS = [
  "2160p", "1080p", "720p", "480p", "4K", "4KDS",
  "WEB-DL", "WEBRip", "WEB", "BluRay", "BRRip", "BDRip", "HDRip", "DVDRip", "HDTV",
  "HEVC", "x265", "x264", "H\\.?264", "H\\.?265", "AV1", "AVC",
  "AAC", "DDP?5\\.1", "5\\.1", "7\\.1", "ATMOS", "AC3", "EAC3", "FLAC", "DTS",
  "10bit", "8bit", "HDR", "HDR10", "SDR", "DV", "Dolby\\.?Vision",
  "REMUX", "REPACK", "PROPER", "EXTENDED", "UNRATED", "IMAX",
  "MUBI", "AMZN", "NF", "DSNP", "HMAX", "ATVP", "PCOK",
];

const STRIP_PATTERN = new RegExp(
  `\\b(?:${STRIP_TOKENS.join("|")})\\b`,
  "gi",
);

// Trailing release group: "-BONE", "-rmteam", or standalone group name after stripping
const RELEASE_GROUP = /[-–]\s*[A-Za-z][A-Za-z0-9]*\s*$/;

// Standalone "tester" or similar suffixes that aren't real words in titles
const TRAILING_JUNK = /\b(?:AV1tester)\b/gi;

// Names where dots serve as word separators (scene releases, mixed-format names)
function hasDotSeparators(name: string): boolean {
  const dots = (name.match(/\./g) || []).length;
  return dots > 2;
}

export function cleanTorrentName(raw: string): string {
  let name = raw;

  // 1. Strip site prefixes
  name = name.replace(SITE_PREFIX, "");

  // 2. Strip bracketed tags, but keep ones that look like alternate titles
  //    e.g. [The Last Viking] — contains spaces and no codec-like content
  name = name.replace(BRACKETED_TAGS, (_match, inner: string) => {
    const isCodecTag = STRIP_PATTERN.test(inner) ||
      /^\d[\d.]*$/.test(inner.trim()) ||
      /^YTS/i.test(inner.trim()) ||
      /^MX$/i.test(inner.trim());
    // Reset lastIndex since STRIP_PATTERN has global flag
    STRIP_PATTERN.lastIndex = 0;
    return isCodecTag ? " " : `[${inner}]`;
  });

  // 3. Replace dots/underscores with spaces if dot-separated
  if (hasDotSeparators(name)) {
    name = name.replace(/[._]/g, " ");
  }

  // 4. Strip known tokens
  name = name.replace(STRIP_PATTERN, " ");

  // 5. Strip trailing junk patterns
  name = name.replace(TRAILING_JUNK, " ");

  // 6. Strip release group suffix (hyphenated form)
  name = name.replace(RELEASE_GROUP, "");

  // 7. Collapse spaces, trim
  name = name.replace(/\s{2,}/g, " ").trim();

  // 8. Strip trailing single-word release group (no hyphen) in dot-separated names.
  //    After token stripping, these are orphaned group names like "rmteam", "TURG".
  if (hasDotSeparators(raw)) {
    name = name.replace(/\s+[A-Za-z]+$/, (match) => {
      const word = match.trim();
      // Keep it if it looks like a real title word (mixed case with vowels)
      if (/[a-z]/.test(word) && /[A-Z]/.test(word)) return match;
      // Keep it if it's a year
      if (/^\d{4}$/.test(word)) return match;
      return "";
    });
  }

  // 9. Collapse spaces again, trim
  name = name.replace(/\s{2,}/g, " ").trim();

  // 10. Capitalize first letter for scene-style names
  if (name.length > 0 && hasDotSeparators(raw)) {
    name = name.charAt(0).toUpperCase() + name.slice(1);
  }

  return name;
}

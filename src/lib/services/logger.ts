// Structured frontend logger. Outputs to console with level prefix.
// In production builds, debug/trace are suppressed.

type LogLevel = "trace" | "debug" | "info" | "warn" | "error";

const LEVEL_ORDER: Record<LogLevel, number> = {
  trace: 0,
  debug: 1,
  info: 2,
  warn: 3,
  error: 4,
};

const MIN_LEVEL: LogLevel = import.meta.env.DEV ? "debug" : "info";

function shouldLog(level: LogLevel): boolean {
  return LEVEL_ORDER[level] >= LEVEL_ORDER[MIN_LEVEL];
}

function formatMsg(level: LogLevel, tag: string, msg: string): string {
  const ts = new Date().toISOString().slice(11, 23);
  return `[${ts}] ${level.toUpperCase().padEnd(5)} [${tag}] ${msg}`;
}

function createLogger(tag: string) {
  return {
    trace(msg: string, ...args: unknown[]) {
      if (shouldLog("trace")) console.debug(formatMsg("trace", tag, msg), ...args);
    },
    debug(msg: string, ...args: unknown[]) {
      if (shouldLog("debug")) console.debug(formatMsg("debug", tag, msg), ...args);
    },
    info(msg: string, ...args: unknown[]) {
      if (shouldLog("info")) console.info(formatMsg("info", tag, msg), ...args);
    },
    warn(msg: string, ...args: unknown[]) {
      if (shouldLog("warn")) console.warn(formatMsg("warn", tag, msg), ...args);
    },
    error(msg: string, ...args: unknown[]) {
      if (shouldLog("error")) console.error(formatMsg("error", tag, msg), ...args);
    },
  };
}

export const log = createLogger("app");
export { createLogger };

import { useCallback, useEffect, useMemo, useState } from "react";
import { useTranslation } from "react-i18next";
import type { ArchiveOverview, ArchiveSession } from "../types";
import {
  archiveEligibleNow,
  getArchiveOverview,
  setAutoArchive,
  setSessionArchived,
} from "../ipc";

type Filter = "all" | "active" | "archived";

/** 相对时间 hook：刚刚 / n 分钟前 / n 小时前 / n 天前（i18n） */
function useRelTime() {
  const { t } = useTranslation();
  return useCallback(
    (ms: number): string => {
      const mins = Math.max(0, Math.floor((Date.now() - ms) / 60000));
      if (mins < 1) return t("archive.justNow");
      if (mins < 60) return t("archive.minutesAgo", { n: mins });
      const hours = Math.floor(mins / 60);
      if (hours < 24) return t("archive.hoursAgo", { n: hours });
      return t("archive.daysAgo", { n: Math.floor(hours / 24) });
    },
    [t],
  );
}

/**
 * 会话归档区（语义移植自 macOS 版）：自动归档开关 + 期限选择 + 立即归档
 * + 会话列表（按工作区分组，可筛选全部/未归档/已归档，逐条归档/取消归档）。
 */
export function ArchiveSection() {
  const { t } = useTranslation();
  const relTime = useRelTime();
  const [overview, setOverview] = useState<ArchiveOverview | null>(null);
  const [filter, setFilter] = useState<Filter>("all");
  const [busy, setBusy] = useState(false);
  const [msg, setMsg] = useState<string | null>(null);

  const reload = useCallback(async () => {
    try {
      setOverview(await getArchiveOverview());
    } catch {
      // 总览加载失败保持旧数据，下次操作时重试
    }
  }, []);

  useEffect(() => {
    void reload();
  }, [reload]);

  const onToggleAuto = async (enabled: boolean) => {
    setMsg(null);
    try {
      await setAutoArchive(enabled, overview?.auto_archive_threshold ?? "oneWeek");
    } catch (e) {
      setMsg(String(e));
    } finally {
      await reload();
    }
  };

  const onThreshold = async (threshold: string) => {
    setMsg(null);
    try {
      await setAutoArchive(overview?.auto_archive_enabled ?? false, threshold);
    } catch (e) {
      setMsg(String(e));
    } finally {
      await reload();
    }
  };

  const onArchiveNow = async () => {
    setBusy(true);
    setMsg(null);
    try {
      const count = await archiveEligibleNow();
      setMsg(t("archive.archivedResult", { count }));
    } catch (e) {
      setMsg(String(e));
    } finally {
      setBusy(false);
    }
    await reload();
  };

  const onSetArchived = async (path: string, archived: boolean) => {
    setMsg(null);
    try {
      const ok = await setSessionArchived(path, archived);
      if (!ok) setMsg(t("archive.opFailed"));
    } catch (e) {
      setMsg(String(e));
    } finally {
      await reload();
    }
  };

  const groups = useMemo(() => {
    if (overview === null) return [];
    const filtered = overview.sessions.filter((s) =>
      filter === "all" ? true : filter === "archived" ? s.is_archived : !s.is_archived,
    );
    const map = new Map<string, ArchiveSession[]>();
    for (const s of filtered) {
      const key = `${s.folder_name} (${s.workspace_hash})`;
      const list = map.get(key) ?? [];
      list.push(s);
      map.set(key, list);
    }
    return [...map.entries()];
  }, [overview, filter]);

  if (overview === null) {
    return <p className="muted-text">{t("archive.loading")}</p>;
  }

  return (
    <div className="archive-section">
      <div className="form-row">
        <label htmlFor="archive-auto">{t("archive.auto")}</label>
        <input
          id="archive-auto"
          type="checkbox"
          checked={overview.auto_archive_enabled}
          onChange={(e) => void onToggleAuto(e.target.checked)}
        />
      </div>
      <div className="form-row">
        <label htmlFor="archive-threshold">{t("archive.thresholdLabel")}</label>
        <select
          id="archive-threshold"
          className="input"
          value={overview.auto_archive_threshold}
          onChange={(e) => void onThreshold(e.target.value)}
        >
          <option value="oneDay">{t("archive.oneDay")}</option>
          <option value="oneWeek">{t("archive.oneWeek")}</option>
          <option value="oneMonth">{t("archive.oneMonth")}</option>
        </select>
      </div>
      <div className="archive-toolbar">
        <button className="btn" onClick={() => void onArchiveNow()} disabled={busy}>
          {t("archive.archiveNow")}
        </button>
        <select
          className="input"
          value={filter}
          onChange={(e) => setFilter(e.target.value as Filter)}
        >
          <option value="all">{t("archive.filterAll")}</option>
          <option value="active">{t("archive.filterActive")}</option>
          <option value="archived">{t("archive.filterArchived")}</option>
        </select>
      </div>
      {msg !== null && <p className="hint-ok">{msg}</p>}
      {overview.last_auto_archive_at !== null && (
        <p className="archive-last-auto">
          {t("archive.lastAuto", {
            time: relTime(overview.last_auto_archive_at),
            count: overview.last_auto_archive_count,
          })}
        </p>
      )}
      {overview.error !== null && <p className="hint-err">{overview.error}</p>}
      {groups.length === 0 ? (
        <p className="muted-text">{t("archive.empty")}</p>
      ) : (
        <div className="archive-list">
          {groups.map(([ws, sessions]) => (
            <div key={ws}>
              <div className="archive-ws">
                <span>{ws}</span>
                <span>{sessions.length}</span>
              </div>
              {sessions.map((s) => (
                <div key={s.id} className="archive-row">
                  <div className="archive-row-main">
                    <div className="archive-row-title" title={s.title}>
                      {s.title}
                    </div>
                    <div className="archive-row-meta">
                      {s.is_archived
                        ? t("archive.archivedAt", { time: relTime(s.updated_at_ms) })
                        : t("archive.updatedAt", { time: relTime(s.updated_at_ms) })}
                    </div>
                  </div>
                  <button
                    type="button"
                    className="link"
                    onClick={() => void onSetArchived(s.path, !s.is_archived)}
                  >
                    {s.is_archived ? t("archive.unarchive") : t("archive.archive")}
                  </button>
                </div>
              ))}
            </div>
          ))}
        </div>
      )}
    </div>
  );
}

import { useCallback, useEffect, useState } from "react";
import { toast } from "sonner";
import {
  call,
  hasV050Api,
  type Brief,
  type BriefJob,
  type Decision,
  type Offer,
  type Status,
  type ThreadRow,
  type TodayEvent,
  type TodayPayload,
} from "@/lib/api";

export type PaperData = {
  brief: Brief;
  decisions: Decision[];
  offers: Offer[];
  todayEvents: TodayEvent[];
  threads: ThreadRow[];
  extraReceipts: BriefJob[];
  receiptTotal: number;
};

export function usePaper(onChanged: () => void, status?: Status | null | "error") {
  const [data, setData] = useState<PaperData | null>(null);
  const [failed, setFailed] = useState(false);

  const load = useCallback(async () => {
    const [b, d, o, td, t] = await Promise.all([
      call<Brief>("/api/brief"),
      call<{ decisions: Decision[] }>("/api/decisions"),
      call<{ offers: Offer[] }>("/api/offers"),
      call<TodayPayload>("/api/today"),
      call<{ threads: ThreadRow[] }>("/api/threads"),
    ]);
    const shaped = (x: unknown, key: string) => !!x && typeof x === "object" && Array.isArray((x as Record<string, unknown>)[key]);
    if (
      !b.ok || !d.ok || !o.ok || !td.ok || !t.ok
      || !shaped(b.data, "finished")
      || !shaped(d.data, "decisions")
      || !shaped(o.data, "offers")
      || !shaped(td.data, "events")
      || !shaped(t.data, "threads")
    ) {
      setFailed(true);
      return;
    }

    let extraReceipts: BriefJob[] = [];
    let receiptTotal = 0;
    if (status && hasV050Api(status, "receipts_export")) {
      const rec = await call<{ receipts?: BriefJob[]; total?: number }>("/api/receipts?limit=200");
      if (rec.ok && Array.isArray(rec.data.receipts)) {
        extraReceipts = rec.data.receipts.filter((x): x is BriefJob => !!x && typeof x === "object");
        receiptTotal = typeof rec.data.total === "number" ? rec.data.total : extraReceipts.length;
      }
    }

    setFailed(false);
    setData({
      brief: b.data,
      decisions: (d.data.decisions ?? []).filter((x): x is Decision => !!x && typeof x === "object"),
      offers: (o.data.offers ?? []).filter((x): x is Offer => !!x && typeof x === "object"),
      todayEvents: (td.data.events ?? []).filter((x): x is TodayEvent => !!x && typeof x === "object"),
      threads: (t.data.threads ?? []).filter((x): x is ThreadRow => !!x && typeof x === "object"),
      extraReceipts,
      receiptTotal,
    });
  }, [status]);

  const loadAllReceipts = useCallback(async () => {
    const rec = await call<{ receipts?: BriefJob[]; total?: number }>("/api/receipts?limit=5000");
    if (rec.ok && Array.isArray(rec.data.receipts)) {
      const list = rec.data.receipts.filter((x): x is BriefJob => !!x && typeof x === "object");
      setData((prev) =>
        prev
          ? {
              ...prev,
              extraReceipts: list,
              receiptTotal: typeof rec.data.total === "number" ? rec.data.total : list.length,
            }
          : prev
      );
    }
  }, []);

  useEffect(() => { load(); }, [load]);

  const act = useCallback(async (path: string, done: string) => {
    const r = await call<{ outcome?: string }>(path, { method: "POST", body: {} });
    if (!r.ok) {
      toast.error(r.error ? "That did not go through: " + r.error : "That did not go through. Try again.");
      return;
    }
    toast.success(typeof r.data?.outcome === "string" && r.data.outcome ? r.data.outcome : done);
    await load();
    onChanged();
  }, [load, onChanged]);

  return { data, failed, load, loadAllReceipts, act };
}

// Plain words for what the server reports. The raw text stays one tap away in Details.
import type { Update } from "./api";

export function toolLabel(tool: string, via: string | undefined, body: string): string | null {
  const pick = (re: RegExp) => { const m = re.exec(body); return m ? m[1] : ""; };
  let label: string | null;
  switch (tool) {
    case "list_dir": label = "Looked through your files"; break;
    case "read_file": label = "Read a file"; break;
    case "write_file": { const n = pick(/^wrote (.+?) \(\d+ bytes\)/); label = n ? "Saved " + n : "Saved a file"; break; }
    case "write_document": { const n = pick(/^wrote document (.+?) \(/); label = n ? "Saved " + n : "Saved a document"; break; }
    case "make_spreadsheet": { const n = pick(/^wrote spreadsheet (.+?) \(/); label = n ? "Saved spreadsheet " + n : "Saved a spreadsheet"; break; }
    case "move_file": label = "Moved a file"; break;
    case "search_files": label = "Searched your files"; break;
    case "fetch_url": label = "Read a web page"; break;
    case "research": label = "Researched a topic"; break;
    case "update_memory": label = "Remembered something about you"; break;
    case "browse_library": label = "Looked through the company library"; break;
    case "install_item": label = "Added an item to your suite"; break;
    case "publish_skill": label = "Published a skill"; break;
    case "delete_file": { const n = pick(/(\S+) has NOT been deleted/); label = "Asked your approval to delete " + (n || "a file"); break; }
    case "flag_decision": label = "Asked you to decide something"; break;
    case "delegate": label = via ? "Handed work to " + via : "Handed work to a specialist"; break;
    default: return null;
  }
  if (via && tool !== "delegate") label += " (for " + via + ")";
  return label;
}

/** The workspace-relative file a save step wrote. */
export function savedFile(tool: string, body: string): string {
  const re = tool === "write_document" ? /^wrote document (.+?) \(/
    : tool === "make_spreadsheet" ? /^wrote spreadsheet (.+?) \(/
    : tool === "write_file" ? /^wrote (.+?) \(\d+ bytes\)/ : null;
  const m = re && re.exec(body);
  return m ? m[1] : "";
}

export const FAILURE_RE = /not a JSON object|no model key|model endpoint returned HTTP|without finishing|could not reach|could not run|did not finish/i;
export const PARKED_RE = /parked for your person|has NOT been deleted yet/;
export const RESOLVED_RE = /was deleted after you approved it|was kept because you declined/;

export function relTime(ms: number): string {
  const now = new Date(); const then = new Date(ms);
  const mins = Math.round((now.getTime() - then.getTime()) / 60000);
  if (mins < 1) return "just now";
  if (mins < 60) return mins + " min ago";
  const dayNow = new Date(now.getFullYear(), now.getMonth(), now.getDate());
  const dayThen = new Date(then.getFullYear(), then.getMonth(), then.getDate());
  const days = Math.round((dayNow.getTime() - dayThen.getTime()) / 86400000);
  if (days === 0) return Math.round(mins / 60) + " h ago";
  if (days === 1) return "Yesterday";
  return then.toLocaleDateString([], { month: "short", day: "numeric" });
}

export function threadMs(id: string): number | null {
  const m = /^t(\d{13})/.exec(String(id || ""));
  return m ? Number(m[1]) : null;
}

/** Turns the server's update error text into one plain sentence. Raw text is shown only in Details. */
export function plainUpdateError(raw: string): string {
  const r = raw || "";
  if (/could not reach|Connection/i.test(r)) return "Could not reach GitHub. Check your internet connection, then try again.";
  if (/checksum/i.test(r)) return "The download did not pass its safety check, so it was not installed. Nothing changed. Try again in a few minutes.";
  if (/answered HTTP/i.test(r)) return "GitHub had a problem. Nothing changed. Try again in a few minutes.";
  if (/only allowed from this machine/i.test(r)) return "Updating works only on the computer Ergon runs on. Open it there to update.";
  if (/does not run on this machine/i.test(r)) return "The new version does not run on this computer, so it was not installed. Nothing changed.";
  return "The update did not finish. Nothing changed. Try again in a few minutes.";
}

export function resultLine(s: Update): string {
  if (s.mode === "off") return "Update checks are turned off on this Ergon.";
  if (s.error) return plainUpdateError(s.error);
  if (s.staged) return "Version " + s.staged + " installed. Restart to use it.";
  if (s.available) return "Version " + s.latest + " is available";
  if (!s.checked_at) return "Not checked yet.";
  const mins = Math.round((Date.now() - s.checked_at * 1000) / 60000);
  const when = mins < 1 ? "just now" : mins < 60 ? mins + " min ago" : Math.round(mins / 60) + " h ago";
  return "You are on the newest version (checked " + when + ")";
}

export const isLocal = () => ["localhost", "127.0.0.1", "[::1]"].includes(location.hostname);

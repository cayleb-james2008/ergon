// Today and Work Graph are built in round 2; this round only makes them reachable.
export function LaterFace({ face }: { face: "today" | "graph" }) {
  return (
    <section aria-label={face === "today" ? "Today" : "Work graph"} className="flex flex-1 items-start px-6 pt-8">
      <p className="text-ui text-ink-2">{face === "today" ? "Today" : "Work Graph"} is built in the next round. Chat is the working screen for now.</p>
    </section>
  );
}

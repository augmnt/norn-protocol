export function Insight() {
  return (
    <section className="border-t">
      <div className="mx-auto max-w-7xl px-4 py-20 sm:px-6 lg:px-8">
        <div className="max-w-3xl">
          <p className="text-2xl sm:text-3xl leading-relaxed font-light">
            <span className="text-muted-foreground">
              Every blockchain charges you to move your own money. Gas fees,
              base fees, tips — your coffee costs more in transaction fees
              than milk.
            </span>
          </p>
          <p className="mt-8 text-2xl sm:text-3xl leading-relaxed font-light">
            <span className="text-foreground font-normal">
              You sign a transfer. The network validates it. Done.
            </span>{" "}
            <span className="text-muted-foreground">
              Near-zero fees. No waiting. The chain validates state, settles disputes,
              and anchors history — but only your signature controls your money.
            </span>
          </p>
        </div>
      </div>
    </section>
  );
}

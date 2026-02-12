import { ImageResponse } from "next/og";

export const runtime = "edge";
export const alt = "Norn Protocol — The chain is a courtroom, not a bank";
export const size = { width: 1200, height: 630 };
export const contentType = "image/png";

const heroArt = `     │           │           │
     │           │           │
     │           │           │
─────┼─────      │      ─────┼─────
     │      ─────┼─────      │
═════╪═════      │      ═════╪═════
     │      ═════╪═════      │
─────┼─────      │      ─────┼─────
     │      ─────┼─────      │
═════╪═════      │      ═════╪═════
      ╲     ═════╪═════     ╱
       ╲    ─────┼─────    ╱
════════╪════════╪════════╪════════
         ╲  ═════╪═════  ╱
──────────╲──────┼──────╱──────────
           ╲     │     ╱
════════════╪════╪════╪════════════
             ╲   │   ╱
──────────────╲──┼──╱──────────────
               ╲ │ ╱
                ╲│╱
                 ●
                ╱│╲
               ╱ │ ╲
──────────────╱──┼──╲──────────────
             ╱   │   ╲
════════════╪════╪════╪════════════
           ╱     │     ╲
──────────╱──────┼──────╲──────────
         ╱  ═════╪═════  ╲
════════╪════════╪════════╪════════
       ╱    ─────┼─────    ╲
      ╱     ═════╪═════     ╲
═════╪═════      │      ═════╪═════
     │      ─────┼─────      │
─────┼─────      │      ─────┼─────
     │      ═════╪═════      │
═════╪═════      │      ═════╪═════
     │      ─────┼─────      │
─────┼─────      │      ─────┼─────
     │           │           │
     │           │           │
     │           │           │`;

export default async function Image() {
  const inter = await fetch(
    new URL(
      "https://fonts.gstatic.com/s/inter/v20/UcCO3FwrK3iLTeHuS_nVMrMxCp50SjIw2boKoduKmMEVuFuYMZg.ttf"
    )
  ).then((res) => res.arrayBuffer());

  const jetbrains = await fetch(
    new URL(
      "https://fonts.gstatic.com/s/jetbrainsmono/v24/tDbY2o-flEEny0FZhsfKu5WU4zr3E_BX0PnT8RD8-qxjPQ.ttf"
    )
  ).then((res) => res.arrayBuffer());

  return new ImageResponse(
    (
      <div
        style={{
          width: "100%",
          height: "100%",
          display: "flex",
          position: "relative",
          background: "#09090b",
          overflow: "hidden",
        }}
      >
        {/* Top accent line */}
        <div
          style={{
            position: "absolute",
            top: 0,
            left: 0,
            right: 0,
            height: 2,
            background:
              "linear-gradient(90deg, transparent 0%, rgba(120,135,155,0.5) 30%, rgba(120,135,155,0.5) 70%, transparent 100%)",
          }}
        />

        {/* Corner marks */}
        <div
          style={{
            position: "absolute",
            top: 24,
            left: 24,
            width: 20,
            height: 20,
            borderTop: "1px solid rgba(255,255,255,0.06)",
            borderLeft: "1px solid rgba(255,255,255,0.06)",
          }}
        />
        <div
          style={{
            position: "absolute",
            top: 24,
            right: 24,
            width: 20,
            height: 20,
            borderTop: "1px solid rgba(255,255,255,0.06)",
            borderRight: "1px solid rgba(255,255,255,0.06)",
          }}
        />
        <div
          style={{
            position: "absolute",
            bottom: 24,
            left: 24,
            width: 20,
            height: 20,
            borderBottom: "1px solid rgba(255,255,255,0.06)",
            borderLeft: "1px solid rgba(255,255,255,0.06)",
          }}
        />
        <div
          style={{
            position: "absolute",
            bottom: 24,
            right: 24,
            width: 20,
            height: 20,
            borderBottom: "1px solid rgba(255,255,255,0.06)",
            borderRight: "1px solid rgba(255,255,255,0.06)",
          }}
        />

        {/* ASCII art background */}
        <div
          style={{
            position: "absolute",
            right: 40,
            top: "50%",
            transform: "translateY(-50%)",
            display: "flex",
          }}
        >
          <pre
            style={{
              fontFamily: "JetBrains Mono",
              fontSize: 10.5,
              lineHeight: 1.25,
              color: "rgba(140,150,165,0.10)",
              whiteSpace: "pre",
            }}
          >
            {heroArt}
          </pre>
        </div>

        {/* Content */}
        <div
          style={{
            display: "flex",
            flexDirection: "column",
            justifyContent: "center",
            padding: "60px 80px",
            height: "100%",
            position: "relative",
            zIndex: "2",
          }}
        >
          {/* NORN wordmark */}
          <div
            style={{
              fontFamily: "Inter",
              fontSize: 96,
              fontWeight: 700,
              color: "#fafafa",
              letterSpacing: "-0.04em",
              lineHeight: 1,
              marginBottom: 16,
            }}
          >
            norn
          </div>

          {/* Mono label */}
          <div
            style={{
              fontFamily: "JetBrains Mono",
              fontSize: 14,
              fontWeight: 500,
              color: "rgba(120,135,155,0.6)",
              letterSpacing: "0.08em",
              marginBottom: 32,
            }}
          >
            PROTOCOL
          </div>

          {/* Tagline */}
          <div
            style={{
              display: "flex",
              flexDirection: "column",
              fontFamily: "Inter",
              fontSize: 32,
              fontWeight: 700,
              lineHeight: 1.2,
              letterSpacing: "-0.02em",
              maxWidth: 520,
            }}
          >
            <span style={{ color: "rgba(200,205,215,0.7)" }}>
              The chain is a courtroom,
            </span>
            <span style={{ color: "rgba(160,165,175,0.35)" }}>not a bank.</span>
          </div>

          {/* Subtitle */}
          <div
            style={{
              fontFamily: "Inter",
              fontSize: 16,
              fontWeight: 400,
              color: "rgba(160,165,175,0.4)",
              lineHeight: 1.6,
              maxWidth: 440,
              marginTop: 20,
            }}
          >
            A thread-centric blockchain where users own their state.
            Zero-fee transfers, fast finality, cryptographic state verification.
          </div>
        </div>

        {/* URL */}
        <div
          style={{
            position: "absolute",
            bottom: 48,
            left: 80,
            fontFamily: "JetBrains Mono",
            fontSize: 13,
            color: "rgba(120,135,155,0.35)",
          }}
        >
          norn.network
        </div>
      </div>
    ),
    {
      ...size,
      fonts: [
        {
          name: "Inter",
          data: inter,
          style: "normal",
          weight: 700,
        },
        {
          name: "JetBrains Mono",
          data: jetbrains,
          style: "normal",
          weight: 500,
        },
      ],
    }
  );
}

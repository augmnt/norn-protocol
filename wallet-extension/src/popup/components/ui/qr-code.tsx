import { QRCodeSVG } from "qrcode.react";

interface QRCodeDisplayProps {
  value: string;
  size?: number;
}

export function QRCodeDisplay({ value, size = 160 }: QRCodeDisplayProps) {
  return (
    <div className="inline-flex items-center justify-center rounded-lg bg-white p-3">
      <QRCodeSVG
        value={value}
        size={size}
        bgColor="#ffffff"
        fgColor="#09090b"
        level="M"
      />
    </div>
  );
}

import { useState } from "react";
import { Identicon } from "./identicon";

interface NnsAvatarProps {
  address: string;
  avatarUrl?: string | null;
  size?: number;
  className?: string;
}

export function NnsAvatar({ address, avatarUrl, size = 32, className }: NnsAvatarProps) {
  const [imgError, setImgError] = useState(false);

  if (!avatarUrl || imgError) {
    return <Identicon address={address} size={size} className={className} />;
  }

  return (
    <img
      src={avatarUrl}
      alt=""
      width={size}
      height={size}
      className={`rounded-full object-cover ${className ?? ""}`}
      style={{ width: size, height: size }}
      onError={() => setImgError(true)}
    />
  );
}

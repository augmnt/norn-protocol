"use client";

export default function ChatLayout({ children }: { children: React.ReactNode }) {
  return (
    <div className="flex flex-col h-[calc(100dvh-3.5rem)] pb-16 md:pb-0">
      {children}
    </div>
  );
}

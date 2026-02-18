"use client";

export default function ChatLayout({ children }: { children: React.ReactNode }) {
  return (
    <div className="flex flex-col h-[calc(100vh-3.5rem)]">
      {children}
    </div>
  );
}

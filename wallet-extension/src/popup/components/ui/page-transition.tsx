interface PageTransitionProps {
  children: React.ReactNode;
  routeKey: string;
}

export function PageTransition({ children, routeKey }: PageTransitionProps) {
  return (
    <div key={routeKey} className="h-full animate-fade-in">
      {children}
    </div>
  );
}

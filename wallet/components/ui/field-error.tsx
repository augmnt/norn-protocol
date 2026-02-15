interface FieldErrorProps {
  message: string;
  show: boolean;
}

export function FieldError({ message, show }: FieldErrorProps) {
  if (!show) return null;
  return <p className="text-[11px] text-destructive">{message}</p>;
}

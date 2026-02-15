import * as React from "react";
import { Button, type ButtonProps } from "@/components/ui/button";

export interface FormButtonProps extends ButtonProps {
  disabledReason?: string;
}

const FormButton = React.forwardRef<HTMLButtonElement, FormButtonProps>(
  ({ disabledReason, disabled, children, ...props }, ref) => {
    return (
      <div>
        <Button ref={ref} disabled={disabled} {...props}>
          {children}
        </Button>
        {disabled && disabledReason && (
          <p className="mt-1.5 text-center text-[11px] text-muted-foreground">
            {disabledReason}
          </p>
        )}
      </div>
    );
  }
);
FormButton.displayName = "FormButton";

export { FormButton };

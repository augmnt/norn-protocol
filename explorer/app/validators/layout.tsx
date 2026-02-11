import type { Metadata } from "next";

export const metadata: Metadata = {
  title: "Validators",
  description: "View validators and staking information on the Norn network.",
};

export default function ValidatorsLayout({ children }: { children: React.ReactNode }) {
  return children;
}

import { Hero } from "@/components/landing/hero";
import { Features } from "@/components/landing/features";
import { ArchitectureDiagram } from "@/components/landing/architecture-diagram";
import { Ecosystem } from "@/components/landing/ecosystem";
import { Tokenomics } from "@/components/landing/tokenomics";
import { CtaSection } from "@/components/landing/cta-section";

export default function HomePage() {
  return (
    <>
      <Hero />
      <Features />
      <ArchitectureDiagram />
      <Ecosystem />
      <Tokenomics />
      <CtaSection />
    </>
  );
}

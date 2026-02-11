import { Hero } from "@/components/landing/hero";
import { Insight } from "@/components/landing/insight";
import { Stats } from "@/components/landing/stats";
import { HowItWorks } from "@/components/landing/how-it-works";
import { CodeExamples } from "@/components/landing/code-examples";
import { Comparison } from "@/components/landing/comparison";
import { Ecosystem } from "@/components/landing/ecosystem";
import { Tokenomics } from "@/components/landing/tokenomics";
import { CtaSection } from "@/components/landing/cta-section";

export default function HomePage() {
  return (
    <>
      <Hero />
      <Insight />
      <Stats />
      <HowItWorks />
      <CodeExamples />
      <Comparison />
      <Ecosystem />
      <Tokenomics />
      <CtaSection />
    </>
  );
}

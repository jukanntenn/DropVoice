import { useEffect } from 'react';
import { useTranslation } from 'react-i18next';

import { LiquidBackground, useTheme } from '@dropvoice/ui';

import { Nav } from './components/Nav';
import { FaqSection } from './sections/FaqSection';
import { FactsLine } from './sections/FactsLine';
import { FinalCta } from './sections/FinalCta';
import { Footer } from './sections/Footer';
import { Hero } from './sections/Hero';
import { HowItWorks } from './sections/HowItWorks';
import { MultiDeviceSection } from './sections/MultiDeviceSection';
import { PrivacySection } from './sections/PrivacySection';
import { SelfHostSection } from './sections/SelfHostSection';

export default function App() {
  const { i18n } = useTranslation();
  useTheme();

  useEffect(() => {
    document.documentElement.lang = i18n.language;
  }, [i18n.language]);

  return (
    <div id="top" className="min-h-screen">
      <LiquidBackground />
      <Nav />
      <main>
        <Hero />
        <FactsLine />
        <HowItWorks />
        <PrivacySection />
        <MultiDeviceSection />
        <SelfHostSection />
        <FaqSection />
        <FinalCta />
      </main>
      <Footer />
    </div>
  );
}

import { useTranslation } from 'react-i18next';

const FAQ_KEYS = [1, 2, 3, 4, 5] as const;

/**
 * 紧凑诚实 FAQ（设计定稿）：无手风琴，全部直接可见；每条 ≤2 行，
 * 只回答怀疑型访客的真实疑虑，答得诚实——包括局限。
 */
export function FaqSection() {
  const { t } = useTranslation();

  return (
    <section id="faq">
      <div className="mx-auto max-w-3xl px-4 py-20 md:py-28">
        <h2 className="text-2xl font-semibold tracking-tight text-neutral-900 md:text-3xl dark:text-white">
          {t('landing:faq.title')}
        </h2>

        <dl className="mt-10 flex flex-col gap-8">
          {FAQ_KEYS.map((n) => (
            <div key={n}>
              <dt className="font-medium text-neutral-900 dark:text-white">
                {t(`landing:faq.q${n}`)}
              </dt>
              <dd className="mt-1.5 text-sm leading-relaxed text-neutral-600 dark:text-neutral-300">
                {t(`landing:faq.a${n}`)}
              </dd>
            </div>
          ))}
        </dl>
      </div>
    </section>
  );
}

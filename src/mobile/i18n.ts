import i18n from "i18next";
import { initReactI18next } from "react-i18next";

const resources = {
  en: {
    translation: {
      status: {
        connected: "Connected",
        disconnected: "Disconnected",
        connecting: "Connecting...",
        reconnecting: "Reconnecting...",
        connectionLost: "Connection lost. Refresh to try again.",
      },
      input: {
        placeholder: "Type text to send to your PC...",
        restore: "Restore Last Message",
        charCount: "{{count}} / 10000",
        textTooLong: "Text too long (max 10000 characters)",
      },
      actions: {
        send: "Send to PC",
        sending: "Sending...",
      },
      notifications: {
        sent: "Text sent successfully!",
        lastMessageRestored: "Last message restored",
        sendFailed: "Failed to send text",
      },
      settings: {
        title: "Settings",
        language: "Language",
        theme: "Theme",
        themeSystem: "System",
        themeLight: "Light",
        themeDark: "Dark",
      },
    },
  },
  zh: {
    translation: {
      status: {
        connected: "已连接",
        disconnected: "未连接",
        connecting: "连接中...",
        reconnecting: "重连中...",
        connectionLost: "连接已断开，请刷新页面重试",
      },
      input: {
        placeholder: "输入要发送到电脑的文字...",
        restore: "恢复上一条消息",
        charCount: "{{count}} / 10000",
        textTooLong: "文本过长（最多10000字符）",
      },
      actions: {
        send: "发送到电脑",
        sending: "发送中...",
      },
      notifications: {
        sent: "发送成功！",
        lastMessageRestored: "已恢复上一条消息",
        sendFailed: "发送失败",
      },
      settings: {
        title: "设置",
        language: "语言",
        theme: "主题",
        themeSystem: "跟随系统",
        themeLight: "浅色",
        themeDark: "深色",
      },
    },
  },
} as const;

function detectLanguage(): "en" | "zh" {
  const browserLang = navigator.language || (navigator as any).userLanguage;
  if (typeof browserLang === "string" && browserLang.toLowerCase().startsWith("zh")) {
    return "zh";
  }
  return "en";
}

function getInitialLanguage(): "en" | "zh" {
  const saved = localStorage.getItem("dropvoice-mobile-lang");
  if (saved === "en" || saved === "zh") return saved;
  return detectLanguage();
}

export async function initMobileI18n() {
  await i18n.use(initReactI18next).init({
    resources: resources as any,
    lng: getInitialLanguage(),
    fallbackLng: "en",
    interpolation: {
      escapeValue: false,
    },
    react: {
      useSuspense: false,
    },
  });
}

export function setMobileLanguage(lang: "en" | "zh") {
  localStorage.setItem("dropvoice-mobile-lang", lang);
  i18n.changeLanguage(lang);
}

export default i18n;

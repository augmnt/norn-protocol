const LOCKED_KEY = "norn_locked";
const KEYSTORE_KEY = "norn_keystore";
const AUTO_LOCK_ALARM = "auto-lock";
const KEEP_ALIVE_ALARM = "keep-alive";

chrome.runtime.onInstalled.addListener(() => {
  chrome.storage.local.set({ [LOCKED_KEY]: true });
  chrome.alarms.create(KEEP_ALIVE_ALARM, { periodInMinutes: 0.5 });
});

chrome.alarms.onAlarm.addListener((alarm) => {
  if (alarm.name === AUTO_LOCK_ALARM) {
    chrome.storage.local.set({ [LOCKED_KEY]: true });
    chrome.alarms.clear(AUTO_LOCK_ALARM);
  }
});

chrome.runtime.onMessage.addListener((message, _sender, sendResponse) => {
  if (message.type === "GET_LOCK_STATE") {
    chrome.storage.local.get(LOCKED_KEY, (result) => {
      sendResponse({ locked: result[LOCKED_KEY] !== false });
    });
    return true;
  }

  if (message.type === "SET_LOCK_STATE") {
    chrome.storage.local.set({ [LOCKED_KEY]: message.locked });
    if (message.locked) {
      chrome.alarms.clear(AUTO_LOCK_ALARM);
    }
    sendResponse({ ok: true });
    return true;
  }

  if (message.type === "RESET_AUTO_LOCK") {
    chrome.storage.local.get(KEYSTORE_KEY, (result) => {
      const ks = result[KEYSTORE_KEY];
      const minutes = ks?.autoLockMinutes ?? 15;
      chrome.alarms.clear(AUTO_LOCK_ALARM, () => {
        chrome.alarms.create(AUTO_LOCK_ALARM, { delayInMinutes: minutes });
        sendResponse({ ok: true });
      });
    });
    return true;
  }

  if (message.type === "CLEAR_AUTO_LOCK") {
    chrome.alarms.clear(AUTO_LOCK_ALARM);
    sendResponse({ ok: true });
    return true;
  }

  return false;
});

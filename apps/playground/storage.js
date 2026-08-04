const DATABASE_NAME = "geullint-playground";
const STORE_NAME = "local";
const DRAFT_KEY = "draft";
const SETTINGS_KEY = "settings";
const DICTIONARY_KEY = "dictionary";

export function createLocalStore({
  databaseName = DATABASE_NAME,
  storeName = STORE_NAME,
  draftKey = DRAFT_KEY,
} = {}) {
  const canUseIndexedDb = typeof indexedDB !== "undefined";
  const canUseLocalStorage = typeof localStorage !== "undefined";

  async function database() {
    if (!canUseIndexedDb) return undefined;
    return new Promise((resolve, reject) => {
      const request = indexedDB.open(databaseName, 1);
      request.onupgradeneeded = () => request.result.createObjectStore(storeName);
      request.onsuccess = () => resolve(request.result);
      request.onerror = () => reject(request.error ?? new Error("IndexedDB를 열 수 없습니다."));
    });
  }

  async function read(key) {
    if (canUseIndexedDb) {
      const db = await database();
      return new Promise((resolve, reject) => {
        const request = db.transaction(storeName, "readonly").objectStore(storeName).get(key);
        request.onsuccess = () => resolve(request.result);
        request.onerror = () => reject(request.error ?? new Error("로컬 데이터를 읽을 수 없습니다."));
      });
    }
    if (canUseLocalStorage) return localStorage.getItem(key) ?? undefined;
    return undefined;
  }

  async function write(key, value) {
    if (canUseIndexedDb) {
      const db = await database();
      return new Promise((resolve, reject) => {
        const transaction = db.transaction(storeName, "readwrite");
        transaction.objectStore(storeName).put(value, key);
        transaction.oncomplete = () => resolve();
        transaction.onerror = () => reject(transaction.error ?? new Error("로컬 데이터를 저장할 수 없습니다."));
      });
    }
    if (canUseLocalStorage) localStorage.setItem(key, String(value));
  }

  async function loadJson(key, fallback) {
    const value = await read(key);
    if (value === undefined) return fallback;
    try {
      return JSON.parse(String(value));
    } catch {
      return fallback;
    }
  }

  async function saveJson(key, value) {
    return write(key, JSON.stringify(value));
  }

  return {
    async loadDraft() {
      const value = await read(draftKey);
      return typeof value === "string" ? value : undefined;
    },
    async saveDraft(text) {
      return write(draftKey, String(text));
    },
    async clearDraft() {
      if (canUseIndexedDb) {
        const db = await database();
        return new Promise((resolve, reject) => {
          const transaction = db.transaction(storeName, "readwrite");
          transaction.objectStore(storeName).delete(draftKey);
          transaction.oncomplete = () => resolve();
          transaction.onerror = () => reject(transaction.error ?? new Error("초안을 삭제할 수 없습니다."));
        });
      }
      if (canUseLocalStorage) localStorage.removeItem(draftKey);
    },
    async loadSettings() {
      return loadJson(SETTINGS_KEY, {});
    },
    async saveSettings(settings) {
      return saveJson(SETTINGS_KEY, settings);
    },
    async loadDictionary() {
      const value = await loadJson(DICTIONARY_KEY, []);
      return Array.isArray(value) ? value.filter((entry) => typeof entry === "string") : [];
    },
    async saveDictionary(dictionary) {
      return saveJson(DICTIONARY_KEY, dictionary);
    },
  };
}

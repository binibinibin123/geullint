const DATABASE_NAME = "geullint-playground";
const STORE_NAME = "local";
const DRAFT_KEY = "draft";

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

  return {
    async loadDraft() {
      if (canUseIndexedDb) {
        const db = await database();
        return new Promise((resolve, reject) => {
          const request = db.transaction(storeName, "readonly").objectStore(storeName).get(draftKey);
          request.onsuccess = () => resolve(typeof request.result === "string" ? request.result : undefined);
          request.onerror = () => reject(request.error ?? new Error("초안을 읽을 수 없습니다."));
        });
      }
      if (canUseLocalStorage) return localStorage.getItem(draftKey) ?? undefined;
      return undefined;
    },
    async saveDraft(text) {
      if (canUseIndexedDb) {
        const db = await database();
        return new Promise((resolve, reject) => {
          const transaction = db.transaction(storeName, "readwrite");
          transaction.objectStore(storeName).put(String(text), draftKey);
          transaction.oncomplete = () => resolve();
          transaction.onerror = () => reject(transaction.error ?? new Error("초안을 저장할 수 없습니다."));
        });
      }
      if (canUseLocalStorage) localStorage.setItem(draftKey, String(text));
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
  };
}

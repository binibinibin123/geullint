import assert from "node:assert/strict";
import test from "node:test";

import { createHistory } from "../apps/playground/history.js";
import { createLocalStore } from "../apps/playground/storage.js";

test("history keeps bounded reversible text states without duplicating entries", () => {
  const history = createHistory("첫 문장", 3);
  history.push("둘째 문장");
  history.push("둘째 문장");
  history.push("셋째 문장");
  assert.equal(history.current(), "셋째 문장");
  assert.equal(history.undo(), "둘째 문장");
  assert.equal(history.undo(), "첫 문장");
  assert.equal(history.undo(), undefined);
  history.push("넷째 문장");
  assert.equal(history.current(), "넷째 문장");
});

test("history truncates redo states after a new edit", () => {
  const history = createHistory("a", 4);
  history.push("b");
  history.push("c");
  assert.equal(history.undo(), "b");
  history.push("d");
  assert.equal(history.redo(), undefined);
  assert.deepEqual(history.states(), ["a", "b", "d"]);
});

test("local store exposes draft, settings, and dictionary persistence without a server", () => {
  const store = createLocalStore();
  for (const method of ["loadDraft", "saveDraft", "loadSettings", "saveSettings", "loadDictionary", "saveDictionary"]) {
    assert.equal(typeof store[method], "function");
  }
});

function makeDelayedIndexedDb() {
  const values = new Map();
  let openCount = 0;
  const database = {
    createObjectStore() {
      return {};
    },
    transaction() {
      const transaction = {
        oncomplete: undefined,
        onerror: undefined,
        objectStore() {
          return {
            get(key) {
              const request = { onsuccess: undefined, onerror: undefined, result: undefined };
              setTimeout(() => {
                request.result = values.get(key);
                request.onsuccess?.({ target: request });
              }, 0);
              return request;
            },
            put(value, key) {
              values.set(key, value);
            },
            delete(key) {
              values.delete(key);
            },
          };
        },
      };
      setTimeout(() => transaction.oncomplete?.(), 0);
      return transaction;
    },
  };
  return {
    open() {
      const request = { onupgradeneeded: undefined, onsuccess: undefined, onerror: undefined, result: database };
      const delay = openCount++ === 0 ? 100 : 10;
      setTimeout(() => {
        request.onupgradeneeded?.({ target: request });
        request.onsuccess?.({ target: request });
      }, delay);
      return request;
    },
  };
}

function makeLocalStorage() {
  const values = new Map();
  return {
    getItem(key) {
      return values.get(key) ?? null;
    },
    setItem(key, value) {
      values.set(key, String(value));
    },
    removeItem(key) {
      values.delete(key);
    },
  };
}

test("dictionary writes are visible across an immediate reload while IndexedDB is opening", async () => {
  const previousIndexedDb = globalThis.indexedDB;
  const previousLocalStorage = globalThis.localStorage;
  globalThis.indexedDB = makeDelayedIndexedDb();
  globalThis.localStorage = makeLocalStorage();
  try {
    const store = createLocalStore({ databaseName: "geullint-playground-test" });
    const pendingWrite = store.saveDictionary(["product-name"]);
    assert.deepEqual(await store.loadDictionary(), ["product-name"]);
    await pendingWrite;
  } finally {
    globalThis.indexedDB = previousIndexedDb;
    globalThis.localStorage = previousLocalStorage;
  }
});

test("clearing a draft removes the synchronous mirror as well as IndexedDB data", async () => {
  const previousIndexedDb = globalThis.indexedDB;
  const previousLocalStorage = globalThis.localStorage;
  globalThis.indexedDB = makeDelayedIndexedDb();
  globalThis.localStorage = makeLocalStorage();
  try {
    const store = createLocalStore({ databaseName: "geullint-playground-clear-test" });
    const pendingWrite = store.saveDraft("unsaved draft");
    await store.clearDraft();
    assert.equal(await store.loadDraft(), undefined);
    await pendingWrite;
  } finally {
    globalThis.indexedDB = previousIndexedDb;
    globalThis.localStorage = previousLocalStorage;
  }
});

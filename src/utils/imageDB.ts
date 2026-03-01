/**
 * imageDB.ts – IndexedDB helpers for persisting the user-uploaded image.
 *
 * We store the raw Blob (instead of a base-64 data URL) so the DB stays compact.
 * A single record with the fixed key `'current'` is maintained; a new upload
 * simply overwrites the previous record.
 *
 * Usage
 *   import { saveImageToIDB, loadImageFromIDB, clearImageFromIDB } from './imageDB';
 */

const DB_NAME = 'posesolve-images';
const DB_VERSION = 1;
const STORE_NAME = 'images';
const CURRENT_KEY = 'current';

export interface StoredImage {
  blob: Blob;
  name: string;
  width: number;
  height: number;
}

// ─── Internal helpers ─────────────────────────────────────────────────────────

function openDB(): Promise<IDBDatabase> {
  return new Promise((resolve, reject) => {
    const req = indexedDB.open(DB_NAME, DB_VERSION);

    req.onupgradeneeded = () => {
      const db = req.result;
      if (!db.objectStoreNames.contains(STORE_NAME)) {
        db.createObjectStore(STORE_NAME);
      }
    };

    req.onsuccess = () => resolve(req.result);
    req.onerror = () => reject(req.error);
  });
}

// ─── Public API ───────────────────────────────────────────────────────────────

/**
 * Persist a File/Blob together with its known pixel dimensions.
 * The previous record (if any) is overwritten.
 */
export async function saveImageToIDB(
  blob: Blob,
  name: string,
  width: number,
  height: number,
): Promise<void> {
  const db = await openDB();
  return new Promise((resolve, reject) => {
    const tx = db.transaction(STORE_NAME, 'readwrite');
    const store = tx.objectStore(STORE_NAME);
    const record: StoredImage = { blob, name, width, height };
    const req = store.put(record, CURRENT_KEY);
    req.onsuccess = () => resolve();
    req.onerror = () => reject(req.error);
    tx.oncomplete = () => db.close();
  });
}

/**
 * Retrieve the stored image record, or `null` when nothing has been saved yet.
 */
export async function loadImageFromIDB(): Promise<StoredImage | null> {
  const db = await openDB();
  return new Promise((resolve, reject) => {
    const tx = db.transaction(STORE_NAME, 'readonly');
    const store = tx.objectStore(STORE_NAME);
    const req = store.get(CURRENT_KEY);
    req.onsuccess = () => {
      db.close();
      resolve((req.result as StoredImage) ?? null);
    };
    req.onerror = () => {
      db.close();
      reject(req.error);
    };
  });
}

/**
 * Remove the stored image from IndexedDB (e.g., on explicit reset).
 */
export async function clearImageFromIDB(): Promise<void> {
  const db = await openDB();
  return new Promise((resolve, reject) => {
    const tx = db.transaction(STORE_NAME, 'readwrite');
    const store = tx.objectStore(STORE_NAME);
    const req = store.delete(CURRENT_KEY);
    req.onsuccess = () => resolve();
    req.onerror = () => reject(req.error);
    tx.oncomplete = () => db.close();
  });
}

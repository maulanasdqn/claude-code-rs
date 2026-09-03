import { pendingImages } from "./stores.js";

let counter = 0;

export function imagesFromPaste(event) {
  const items = event.clipboardData?.items ?? [];
  const files = [...items]
    .filter((item) => item.kind === "file" && item.type.startsWith("image/"))
    .map((item) => item.getAsFile())
    .filter(Boolean);
  if (files.length === 0) return false;
  addImageFiles(files);
  return true;
}

export function addImageFiles(files) {
  for (const file of files) {
    if (!file.type.startsWith("image/")) continue;
    const reader = new FileReader();
    reader.onload = () => {
      const dataUrl = String(reader.result);
      const base64 = dataUrl.slice(dataUrl.indexOf(",") + 1);
      pendingImages.update((images) => [
        ...images,
        { id: `img-${++counter}`, mediaType: file.type, data: base64 },
      ]);
    };
    reader.readAsDataURL(file);
  }
}

export function removeImage(id) {
  pendingImages.update((images) => images.filter((image) => image.id !== id));
}

export function toPayload(images) {
  return images.map(({ mediaType, data }) => ({ mediaType, data }));
}

export function dataUrl(image) {
  return `data:${image.mediaType};base64,${image.data}`;
}

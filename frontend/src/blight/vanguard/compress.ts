export async function compressBlob(blob: Blob, algo: CompressionFormat = "gzip"): Promise<Uint8Array> {
  return collect(blob.stream().pipeThrough(new CompressionStream(algo)));
}

export async function decompressBlob(blob: Blob, algo: CompressionFormat = "gzip"): Promise<Uint8Array> {
  return collect(blob.stream().pipeThrough(new DecompressionStream(algo)));
}

async function collect(stream: ReadableStream<Uint8Array>): Promise<Uint8Array> {
  const chunks: Uint8Array[] = [];
  const reader = stream.getReader();
  while (true) {
    const { done, value } = await reader.read();
    if (done) return new Uint8Array(await new Blob(chunks as BlobPart[]).arrayBuffer());
    chunks.push(value);
  }
}
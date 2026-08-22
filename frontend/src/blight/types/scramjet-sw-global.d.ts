declare global {
  interface ServiceWorkerGlobalScope {
    $scramjetController: {
      shouldRoute(event: FetchEvent): boolean;
      route(event: FetchEvent): Promise<Response>;
    };
  }
}
export {};
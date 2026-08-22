# History

## Origin

Lightspeed Filter Agent, a school content filter, blocked Wikipedia's images, including the rendered math formulas Wikipedia uses for equations. As a (former) avid Wikipedia surfer, I wanted a way around it. That's where this project started.

## Timeline

**March 2024** : Development began.

**Late May 2024** : First working iteration: Bare + Ultraviolet only.

**June–July 2024** : Added Wisp and wsproxy. Forked Ultraviolet to add adblocking. Added multi-tab support. Added a technique (since retired) for embedding a WASM module as a base-91-encoded string directly inside wasm-bindgen-generated JS, avoiding a separate fetch for the `.wasm` file.

**August 2024** : Sites started breaking. I tried extending the adblocking past its original simple assumptions, first by encoding state in query parameters. Dropped once it turned out to badly hurt performance. Then I switched to fragment identifiers, which worked for adblocking but broke real in-page fragment links. I did additional rewriting to fix it but that didn't succeed either. Around the same time, CAPTCHAs stopped working. By this point my Ultraviolet fork had diverged so far it was effectively its own project, but extremely convoluted and messy. I eventually ceased development of the project as the constant site breakage had become too frustrating to keep chasing and a new school year was starting.

**Late April 2026** : Rediscovered the project while going through my folder of unfinished projects and decided to work on it. Updated dependencies, removed the Ultraviolet fork along with nearly all of the client-side code, keeping only the adblock API, and published the server under the name **Multiconduit**. At that stage the project felt like a set of isolated Minecraft conduits (which are kinda useless), which seemed like it could do so much more if they actually worked together. The server itself had worked fine before the dependency update, but a mistake made while updating types to match the new library's API caused any DNS resolution to immediately crash the server under load (Fixed).

**Mid-2026** : With more time toward the end of the semester, I looked into my options again and found Scramjet, which is Ultraviolet's successor. Scramjet exposes hooks directly, which eliminates the need to fork it at all.

## Renaming back to Blight

With Scramjet handling the proxy internals through hooks instead of a fork, the project no longer needs the kind of hacks that sank the original attempt. That means I can give it back its original name: **Blight**, i.e., the blight of Lightspeed Filter Agent. It no longer has anything to do with Lightspeed specifically, but that's where the project began, and now that it's actually going somewhere, reviving the name feels right.
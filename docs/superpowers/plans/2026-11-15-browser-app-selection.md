# Browser-first connection selection

User requires browser access without installing the Android APK, with an explicit browser/app choice. Existing native remote browser server remains the screen/control transport; Android phone-data service remains separate.

Implemented a top-of-Controls selector, default Browser, exposing only the selected connection instructions and QR. The browser branch invokes mobile_configure and presents HTTP URL plus six-digit code. Android branch invokes phone_configure and presents its westplay URI. Selection does not disconnect an existing background data or screen session, so both can be used together.

Verification: production TypeScript/Vite build passed. Isolated actual React component/browser check passed with native-command stubs: default browser, correct server invocation, HTTP/code shown without APK, native URI only in app mode, and switching back preserves the active browser server. Temporary harness removed before packaging. The optimized Windows app rebuild passed; 518 frontend tests passed and lint reported zero errors (17 existing warnings). Updated share ZIP is generated from these verified files.
